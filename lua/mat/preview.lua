local config = require("mat.config")

local M = {}

M.state = {
  src_buf = nil,
  win = nil,
  buf = nil,
  group = nil,
  job_id = nil,
}

---Check if preview window is open and valid
---@return boolean
function M.is_open()
  return M.state.win ~= nil and vim.api.nvim_win_is_valid(M.state.win)
end

---Close the active preview window and clean up resources
function M.close()
  if M.state.job_id then
    pcall(vim.fn.jobstop, M.state.job_id)
    M.state.job_id = nil
  end

  if M.state.group then
    pcall(vim.api.nvim_del_augroup_by_id, M.state.group)
    M.state.group = nil
  end

  if M.state.win and vim.api.nvim_win_is_valid(M.state.win) then
    vim.api.nvim_win_close(M.state.win, true)
  end

  if M.state.buf and vim.api.nvim_buf_is_valid(M.state.buf) then
    pcall(vim.api.nvim_buf_delete, M.state.buf, { force = true })
  end

  M.state.win = nil
  M.state.buf = nil
  M.state.src_buf = nil
end

---Toggle preview window for current buffer
---@param src_buf? integer
function M.toggle(src_buf)
  if M.is_open() then
    M.close()
  else
    M.open(src_buf)
  end
end

---Open live preview split
---@param src_buf? integer
function M.open(src_buf)
  src_buf = src_buf or vim.api.nvim_get_current_buf()

  local binary = config.get_binary()
  if not binary then
    vim.notify(
      "mat.nvim: Backend binary not found.\nRun :MatBuild inside Neovim or 'cargo build --release' in the plugin root.",
      vim.log.levels.ERROR
    )
    return
  end

  -- If preview is already open, just re-render
  if M.is_open() then
    M.state.src_buf = src_buf
    M.render(src_buf)
    return
  end

  M.state.src_buf = src_buf

  local cur_win = vim.api.nvim_get_current_win()

  -- Open right vertical split
  vim.cmd("botright vsplit")
  local win = vim.api.nvim_get_current_win()
  local split_width = math.floor(vim.o.columns * (config.options.preview.width or 0.45))
  vim.api.nvim_win_set_width(win, split_width)

  -- Initial scratch buffer
  local pbuf = vim.api.nvim_create_buf(false, true)
  local name = "mat://preview/" .. vim.fn.fnamemodify(vim.api.nvim_buf_get_name(src_buf), ":t")
  pcall(vim.api.nvim_buf_set_name, pbuf, name)
  vim.api.nvim_win_set_buf(win, pbuf)

  vim.bo[pbuf].buftype = "nofile"
  vim.bo[pbuf].bufhidden = "wipe"
  vim.bo[pbuf].swapfile = false
  vim.bo[pbuf].filetype = "mat-preview"
  vim.wo[win].number = false
  vim.wo[win].relativenumber = false
  vim.wo[win].signcolumn = "no"
  vim.wo[win].foldcolumn = "0"
  vim.wo[win].wrap = true

  M.state.win = win
  M.state.buf = pbuf

  -- Render initial content
  M.render(src_buf)

  -- Return focus to the active editing window
  if vim.api.nvim_win_is_valid(cur_win) then
    vim.api.nvim_set_current_win(cur_win)
  end

  -- Setup auto-update hooks on source buffer
  if config.options.preview.auto_update then
    local group = vim.api.nvim_create_augroup("MatPreview_" .. src_buf, { clear = true })
    M.state.group = group

    vim.api.nvim_create_autocmd({ "BufWritePost", "BufLeave" }, {
      group = group,
      buffer = src_buf,
      callback = function()
        if M.is_open() then
          M.render(src_buf)
        else
          M.close()
        end
      end,
    })

    vim.api.nvim_create_autocmd("WinClosed", {
      group = group,
      pattern = tostring(win),
      callback = function()
        M.close()
      end,
    })
  end
end

---Render source buffer into preview window via mat
---@param src_buf integer
function M.render(src_buf)
  if not M.is_open() then
    return
  end

  local binary = config.get_binary()
  if not binary then
    return
  end

  local win = M.state.win
  local win_width = math.max(20, vim.api.nvim_win_get_width(win))

  -- Retrieve lines from source buffer
  local lines = vim.api.nvim_buf_get_lines(src_buf, 0, -1, false)
  local content = table.concat(lines, "\n")

  -- Cancel running job if any
  if M.state.job_id then
    pcall(vim.fn.jobstop, M.state.job_id)
    M.state.job_id = nil
  end

  -- Create fresh buffer for atomic swap (zero flicker)
  local new_buf = vim.api.nvim_create_buf(false, true)
  vim.bo[new_buf].buftype = "nofile"
  vim.bo[new_buf].bufhidden = "wipe"
  vim.bo[new_buf].swapfile = false
  vim.bo[new_buf].filetype = "mat-preview"

  local chan = vim.api.nvim_open_term(new_buf, {})

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
      vim.schedule(function()
        if M.state.win and vim.api.nvim_win_is_valid(M.state.win) then
          local old_buf = M.state.buf
          local cur_cursor = vim.api.nvim_win_get_cursor(M.state.win)

          vim.api.nvim_win_set_buf(M.state.win, new_buf)
          M.state.buf = new_buf

          -- Attempt to restore scroll/cursor position
          local line_count = vim.api.nvim_buf_line_count(new_buf)
          local row = math.min(cur_cursor[1], math.max(1, line_count))
          pcall(vim.api.nvim_win_set_cursor, M.state.win, { row, cur_cursor[2] })

          if old_buf and vim.api.nvim_buf_is_valid(old_buf) and old_buf ~= new_buf then
            pcall(vim.api.nvim_buf_delete, old_buf, { force = true })
          end
        else
          if vim.api.nvim_buf_is_valid(new_buf) then
            pcall(vim.api.nvim_buf_delete, new_buf, { force = true })
          end
        end
      end)
    end,
  })

  M.state.job_id = job_id
  vim.fn.chansend(job_id, content)
  vim.fn.chanclose(job_id, "stdin")
end

return M
