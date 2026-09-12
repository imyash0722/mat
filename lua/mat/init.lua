local config = require("mat.config")
local preview = require("mat.preview")

local M = {}

M.setup = function(opts)
  config.setup(opts)

  -- Setup keymaps if configured
  local keymaps = config.options.keymaps
  if keymaps then
    if keymaps.float and keymaps.float ~= "" then
      vim.keymap.set("n", keymaps.float, function()
        M.open()
      end, { desc = "Markdown Preview (Float)", silent = true })
    end
    if keymaps.preview and keymaps.preview ~= "" then
      vim.keymap.set("n", keymaps.preview, function()
        preview.toggle()
      end, { desc = "Markdown Live Preview (Split)", silent = true })
    end
  end
end

M.open = function(opts)
  opts = opts or {}
  local binary = config.get_binary()
  if not binary then
    vim.notify(
      "mat.nvim: Backend binary not found.\nRun :MatBuild to compile with cargo.",
      vim.log.levels.ERROR
    )
    return
  end

  local file = opts.file
  local is_temp = false
  if not file or file == "" then
    local buf = vim.api.nvim_get_current_buf()
    local name = vim.api.nvim_buf_get_name(buf)
    if name ~= "" and not vim.bo[buf].modified then
      file = name
    else
      local lines = vim.api.nvim_buf_get_lines(buf, 0, -1, false)
      file = vim.fn.tempname() .. ".md"
      vim.fn.writefile(lines, file)
      is_temp = true
    end
  end

  local cfg_float = config.options.float
  local width = math.floor(vim.o.columns * (cfg_float.width or 0.85))
  local height = math.floor(vim.o.lines * (cfg_float.height or 0.85))
  local row = math.floor((vim.o.lines - height) / 2)
  local col = math.floor((vim.o.columns - width) / 2)

  local buf = vim.api.nvim_create_buf(false, true)
  local win = vim.api.nvim_open_win(buf, true, {
    relative = "editor",
    width = width,
    height = height,
    row = row,
    col = col,
    style = "minimal",
    border = cfg_float.border or "rounded",
    title = " mat preview ",
    title_pos = "center",
  })

  vim.bo[buf].buftype = "nofile"
  vim.bo[buf].bufhidden = "wipe"

  local close_fn = function()
    if is_temp then
      pcall(vim.fn.delete, file)
      is_temp = false
    end
    if vim.api.nvim_win_is_valid(win) then
      vim.api.nvim_win_close(win, true)
    end
  end

  local cmd = { binary, file }
  vim.fn.termopen(cmd, {
    on_exit = function()
      close_fn()
    end,
  })

  vim.cmd("startinsert")

  vim.keymap.set("t", "<Esc><Esc>", close_fn, { buffer = buf, nowait = true })
  vim.keymap.set("n", "q", close_fn, { buffer = buf, nowait = true })
end

M.open_snippet = function(lines)
  local binary = config.get_binary()
  if not binary then
    vim.notify(
      "mat.nvim: Backend binary not found.\nRun :MatBuild to compile with cargo.",
      vim.log.levels.ERROR
    )
    return
  end

  local file = vim.fn.tempname() .. ".md"
  vim.fn.writefile(lines, file)

  local cfg_float = config.options.float
  local width = math.floor(vim.o.columns * (cfg_float.width or 0.85))
  local height = math.floor(vim.o.lines * (cfg_float.height or 0.85))
  local row = math.floor((vim.o.lines - height) / 2)
  local col = math.floor((vim.o.columns - width) / 2)

  local buf = vim.api.nvim_create_buf(false, true)
  local win = vim.api.nvim_open_win(buf, true, {
    relative = "editor",
    width = width,
    height = height,
    row = row,
    col = col,
    style = "minimal",
    border = cfg_float.border or "rounded",
    title = " mat snippet ",
    title_pos = "center",
  })

  vim.bo[buf].buftype = "nofile"
  vim.bo[buf].bufhidden = "wipe"

  local close_fn = function()
    pcall(vim.fn.delete, file)
    if vim.api.nvim_win_is_valid(win) then
      vim.api.nvim_win_close(win, true)
    end
  end

  local cmd = { binary, file }
  vim.fn.termopen(cmd, {
    on_exit = function()
      close_fn()
    end,
  })

  vim.cmd("startinsert")

  vim.keymap.set("t", "<Esc><Esc>", close_fn, { buffer = buf, nowait = true })
  vim.keymap.set("n", "q", close_fn, { buffer = buf, nowait = true })
end

M.preview = function(opts)
  preview.open(opts and opts.buf)
end

M.preview_toggle = function()
  preview.toggle()
end

M.close = function()
  preview.close()
end

M.build = function()
  local root = config.get_plugin_root()
  if vim.fn.executable("cargo") ~= 1 then
    vim.notify("mat.nvim: 'cargo' not found in PATH. Please install Rust to compile.", vim.log.levels.ERROR)
    return
  end

  vim.notify("mat.nvim: Compiling backend binary with 'cargo build --release'...", vim.log.levels.INFO)
  vim.fn.jobstart({ "cargo", "build", "--release" }, {
    cwd = root,
    stdout_buffered = true,
    stderr_buffered = true,
    on_exit = function(_, code)
      if code == 0 then
        vim.notify("mat.nvim: Successfully compiled backend binary!", vim.log.levels.INFO)
      else
        vim.notify("mat.nvim: Failed to compile backend binary (exit code " .. code .. ")", vim.log.levels.ERROR)
      end
    end,
  })
end

return M
