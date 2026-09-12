local config = require("mat.config")

local M = {}

---@class WindowState
---@field orig_buf integer
---@field preview_buf integer
---@field orig_cursor integer[]
---@field win_opts table
---@field group integer?
---@field job_id integer?

---@type table<integer, WindowState>
M.window_state = {}

---Check if a buffer is a Markdown file
---@param buf integer
---@return boolean
function M.is_markdown(buf)
  if not vim.api.nvim_buf_is_valid(buf) then
    return false
  end
  local ft = vim.bo[buf].filetype
  if ft == "markdown" or ft == "md" then
    return true
  end
  local name = vim.api.nvim_buf_get_name(buf)
  if name:match("%.md$") or name:match("%.markdown$") then
    return true
  end
  return false
end

---Check if a window is currently displaying preview mode
---@param win? integer
---@return boolean
function M.is_preview_mode(win)
  win = win or vim.api.nvim_get_current_win()
  local state = M.window_state[win]
  if state and state.preview_buf and vim.api.nvim_win_is_valid(win) then
    return vim.api.nvim_win_get_buf(win) == state.preview_buf
  end
  return false
end

---Toggle between Preview Mode and Edit Mode in the specified window
---@param win? integer
function M.toggle(win)
  win = win or vim.api.nvim_get_current_win()
  if M.is_preview_mode(win) then
    M.to_edit_mode(win)
  else
    M.to_preview_mode(win)
  end
end

---Switch from Edit Mode to Preview Mode
---@param win? integer
function M.to_preview_mode(win)
  win = win or vim.api.nvim_get_current_win()
  if not vim.api.nvim_win_is_valid(win) then
    return
  end

  if M.is_preview_mode(win) then
    return
  end

  local orig_buf = vim.api.nvim_win_get_buf(win)
  if not M.is_markdown(orig_buf) then
    vim.notify("mat.nvim: Current buffer is not a Markdown file (.md)", vim.log.levels.WARN)
    return
  end

  local binary = config.get_binary()
  if not binary then
    vim.notify(
      "mat.nvim: Backend binary not found.\nRun :MatBuild inside Neovim or 'cargo build --release' in the plugin root.",
      vim.log.levels.ERROR
    )
    return
  end

  -- Record cursor and window display options
  local orig_cursor = vim.api.nvim_win_get_cursor(win)
  local win_opts = {
    number = vim.wo[win].number,
    relativenumber = vim.wo[win].relativenumber,
    signcolumn = vim.wo[win].signcolumn,
    foldcolumn = vim.wo[win].foldcolumn,
    wrap = vim.wo[win].wrap,
  }

  local win_width = math.max(20, vim.api.nvim_win_get_width(win))

  -- Get buffer contents (in-memory, works even with unsaved changes)
  local lines = vim.api.nvim_buf_get_lines(orig_buf, 0, -1, false)
  local content = table.concat(lines, "\n")
  local olines = math.max(1, #lines)
  local ratio = orig_cursor[1] / olines

  -- Create scratch preview buffer
  local pbuf = vim.api.nvim_create_buf(false, true)
  vim.bo[pbuf].buftype = "nofile"
  vim.bo[pbuf].bufhidden = "wipe"
  vim.bo[pbuf].swapfile = false
  vim.bo[pbuf].filetype = "mat-preview"

  local orig_name = vim.fn.fnamemodify(vim.api.nvim_buf_get_name(orig_buf), ":t")
  if orig_name == "" then
    orig_name = "untitled.md"
  end
  pcall(vim.api.nvim_buf_set_name, pbuf, "mat://preview/" .. orig_name)

  local chan = vim.api.nvim_open_term(pbuf, {})

  -- Swap preview buffer into window immediately
  vim.api.nvim_win_set_buf(win, pbuf)

  -- Apply clean viewer window options
  vim.wo[win].number = false
  vim.wo[win].relativenumber = false
  vim.wo[win].signcolumn = "no"
  vim.wo[win].foldcolumn = "0"
  vim.wo[win].wrap = true

  local group = vim.api.nvim_create_augroup("MatToggle_" .. win, { clear = true })

  local job_id = vim.fn.jobstart({ binary, "-p", "-w", tostring(win_width), "-" }, {
    stdout_buffered = true,
    on_stdout = function(_, data)
      if data and chan then
        for i, line in ipairs(data) do
          if i < #data or line ~= "" then
            vim.api.nvim_chan_send(chan, line .. "\r\n")
          end
        end
      end
    end,
    on_exit = function()
      if vim.api.nvim_win_is_valid(win) and vim.api.nvim_win_get_buf(win) == pbuf then
        local plines = math.max(1, vim.api.nvim_buf_line_count(pbuf))
        local target_row = math.max(1, math.min(math.floor(ratio * plines), plines))
        pcall(vim.api.nvim_win_set_cursor, win, { target_row, 0 })
      end
    end,
  })

  M.window_state[win] = {
    orig_buf = orig_buf,
    preview_buf = pbuf,
    orig_cursor = orig_cursor,
    win_opts = win_opts,
    group = group,
    job_id = job_id,
  }

  -- Keybindings inside Preview Mode
  local function bind(key, cb)
    vim.keymap.set("n", key, cb, { buffer = pbuf, nowait = true, silent = true })
  end

  -- i / a: switch back to edit mode and enter insert mode
  bind("i", function()
    M.to_edit_mode(win, { insert = true })
  end)
  bind("a", function()
    M.to_edit_mode(win, { insert = true, append = true })
  end)

  -- e / q / <Esc>: switch back to edit mode in normal mode
  bind("e", function()
    M.to_edit_mode(win)
  end)
  bind("q", function()
    M.to_edit_mode(win)
  end)
  bind("<Esc>", function()
    M.to_edit_mode(win)
  end)

  -- Custom toggle keymap
  local toggle_key = config.options.keymaps and config.options.keymaps.toggle
  if toggle_key and toggle_key ~= "" then
    bind(toggle_key, function()
      M.to_edit_mode(win)
    end)
  end

  -- Auto-cleanup if window closes
  vim.api.nvim_create_autocmd("WinClosed", {
    group = group,
    pattern = tostring(win),
    callback = function()
      M.cleanup(win)
    end,
  })
end

---Switch from Preview Mode back to Edit Mode
---@param win? integer
---@param opts? { insert?: boolean, append?: boolean }
function M.to_edit_mode(win, opts)
  win = win or vim.api.nvim_get_current_win()
  opts = opts or {}
  local state = M.window_state[win]
  if not state then
    return
  end

  -- Stop ongoing job if still streaming
  if state.job_id then
    pcall(vim.fn.jobstop, state.job_id)
    state.job_id = nil
  end

  -- Delete autocmd group
  if state.group then
    pcall(vim.api.nvim_del_augroup_by_id, state.group)
    state.group = nil
  end

  if vim.api.nvim_win_is_valid(win) and vim.api.nvim_buf_is_valid(state.orig_buf) then
    -- Calculate relative cursor from preview buffer
    local target_row = state.orig_cursor[1]
    if state.preview_buf and vim.api.nvim_buf_is_valid(state.preview_buf) then
      local pcur = vim.api.nvim_win_get_cursor(win)
      local plines = math.max(1, vim.api.nvim_buf_line_count(state.preview_buf))
      local ratio = pcur[1] / plines
      local olines = math.max(1, vim.api.nvim_buf_line_count(state.orig_buf))
      target_row = math.max(1, math.min(math.floor(ratio * olines), olines))
    end

    -- Restore original buffer
    vim.api.nvim_win_set_buf(win, state.orig_buf)

    -- Restore window options
    if state.win_opts then
      vim.wo[win].number = state.win_opts.number
      vim.wo[win].relativenumber = state.win_opts.relativenumber
      vim.wo[win].signcolumn = state.win_opts.signcolumn
      vim.wo[win].foldcolumn = state.win_opts.foldcolumn
      vim.wo[win].wrap = state.win_opts.wrap
    end

    -- Restore cursor position
    pcall(vim.api.nvim_win_set_cursor, win, { target_row, state.orig_cursor[2] or 0 })

    -- Switch to insert mode if requested
    if opts.insert then
      if opts.append then
        vim.cmd("startinsert!")
      else
        vim.cmd("startinsert")
      end
    end
  end

  -- Delete preview buffer
  if state.preview_buf and vim.api.nvim_buf_is_valid(state.preview_buf) then
    pcall(vim.api.nvim_buf_delete, state.preview_buf, { force = true })
  end

  M.window_state[win] = nil
end

---Clean up state for closed window
---@param win integer
function M.cleanup(win)
  local state = M.window_state[win]
  if state then
    if state.job_id then
      pcall(vim.fn.jobstop, state.job_id)
    end
    if state.group then
      pcall(vim.api.nvim_del_augroup_by_id, state.group)
    end
    if state.preview_buf and vim.api.nvim_buf_is_valid(state.preview_buf) then
      pcall(vim.api.nvim_buf_delete, state.preview_buf, { force = true })
    end
    M.window_state[win] = nil
  end
end

---Setup auto preview for markdown files if enabled in config
function M.setup_auto_preview()
  local group = vim.api.nvim_create_augroup("MatAutoPreview", { clear = true })
  vim.api.nvim_create_autocmd("BufReadPost", {
    group = group,
    pattern = { "*.md", "*.markdown" },
    callback = function(args)
      vim.schedule(function()
        local cur_win = vim.api.nvim_get_current_win()
        if vim.api.nvim_win_get_buf(cur_win) == args.buf then
          M.to_preview_mode(cur_win)
        end
      end)
    end,
  })
end

return M
