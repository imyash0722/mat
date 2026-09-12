local M = {}

function M.check()
  vim.health.start("mat.nvim")

  -- 1. Check Neovim version
  if vim.fn.has("nvim-0.8.0") == 1 then
    vim.health.ok("Neovim >= 0.8.0 (" .. tostring(vim.version()) .. ")")
  else
    vim.health.warn("Neovim version is older than 0.8.0. Some terminal features may require >= 0.8.")
  end

  -- 2. Check compiled backend binary
  local config = require("mat.config")
  local binary = config.get_binary()
  if binary then
    vim.health.ok("Rust backend binary found: " .. binary)
  else
    vim.health.error(
      "Rust backend binary not found!\n"
        .. "  Run :MatBuild inside Neovim, or execute 'cargo build --release' in the plugin root."
    )
  end

  -- 3. Check cargo / Rust toolchain
  if vim.fn.executable("cargo") == 1 then
    local handle = io.popen("cargo --version 2>/dev/null")
    local ver = handle and handle:read("*a") or ""
    if handle then
      handle:close()
    end
    vim.health.ok("Rust toolchain available: " .. vim.trim(ver))
  else
    vim.health.warn("Cargo not found in PATH. You will need Cargo to build the binary from source.")
  end
end

return M
