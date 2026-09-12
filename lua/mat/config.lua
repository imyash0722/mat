local M = {}

M.defaults = {
  binary = nil, -- custom path to mat backend binary (optional, auto-detected if nil)
  auto_preview = false, -- automatically enter preview mode when opening a .md file
  keymaps = {
    toggle = "<leader>mp", -- Toggle between Preview Mode and Edit Mode in-place
    split = "<leader>ms",  -- Open live side-by-side preview split
    float = "<leader>mv",  -- Open interactive floating preview modal
  },
  float = {
    width = 0.85,       -- width ratio of editor (0.1 - 1.0)
    height = 0.85,      -- height ratio of editor (0.1 - 1.0)
    border = "rounded", -- "none" | "single" | "double" | "rounded" | "solid" | "shadow"
  },
  preview = {
    width = 0.45,       -- split width ratio (0.1 - 1.0)
    auto_update = true, -- live re-render on buffer save
    debounce_ms = 150,  -- debounce delay in milliseconds
  },
}

M.options = vim.deepcopy(M.defaults)

---Find root directory of this plugin
---@return string
function M.get_plugin_root()
  local info = debug.getinfo(1, "S").source
  if info:sub(1, 1) == "@" then
    info = info:sub(2)
  end
  return vim.fs.dirname(vim.fs.dirname(vim.fs.dirname(info)))
end

---Locate the compiled backend binary
---@return string? path
function M.get_binary()
  if M.options.binary and vim.fn.executable(M.options.binary) == 1 then
    return M.options.binary
  end

  local root = M.get_plugin_root()
  local candidates = {
    root .. "/target/release/mat",
    root .. "/target/debug/mat",
    vim.fn.expand("~/.cache/cargo-target/mat/release/mat"),
    vim.fn.expand("~/.local/bin/mat"),
    "mat",
  }

  for _, candidate in ipairs(candidates) do
    if vim.fn.executable(candidate) == 1 then
      return candidate
    end
  end

  return nil
end

function M.setup(opts)
  M.options = vim.tbl_deep_extend("force", {}, M.defaults, opts or {})
end

return M
