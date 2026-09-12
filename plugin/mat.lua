if vim.g.loaded_mat == 1 then
  return
end
vim.g.loaded_mat = 1

-- :Mat [file] or :'<,'>Mat
vim.api.nvim_create_user_command("Mat", function(args)
  if args.range == 2 then
    local lines = vim.api.nvim_buf_get_lines(0, args.line1 - 1, args.line2, false)
    require("mat").open_snippet(lines)
  elseif args.args ~= "" then
    require("mat").open({ file = args.args })
  else
    -- In-place toggle for active Markdown file
    require("mat").toggle()
  end
end, {
  nargs = "?",
  complete = "file",
  range = true,
  desc = "Toggle Markdown Preview/Edit mode, or preview file/selection",
})

-- :MatToggle - In-place flip between preview and edit mode
vim.api.nvim_create_user_command("MatToggle", function()
  require("mat").toggle()
end, {
  desc = "Toggle in-place Markdown Preview and Edit modes in the current window",
})

-- :MatEdit - Switch back to normal edit mode
vim.api.nvim_create_user_command("MatEdit", function()
  require("mat").edit_mode()
end, {
  desc = "Switch current window back to normal Markdown edit mode",
})

-- :MatPreview - Switch current window to rendered preview mode
vim.api.nvim_create_user_command("MatPreview", function()
  require("mat").preview_mode()
end, {
  desc = "Switch current window into rendered Markdown preview mode",
})

-- :MatSplit - Open live side-by-side split preview
vim.api.nvim_create_user_command("MatSplit", function()
  require("mat").preview()
end, {
  desc = "Open live Markdown preview in a side-by-side split",
})

-- :MatFloat - Open floating modal preview
vim.api.nvim_create_user_command("MatFloat", function(args)
  require("mat").open({ file = args.args })
end, {
  nargs = "?",
  complete = "file",
  desc = "Open interactive Markdown preview in a floating modal",
})

-- :MatClose - Close preview in window or split
vim.api.nvim_create_user_command("MatClose", function()
  require("mat").close()
end, {
  desc = "Close active Markdown preview (in-place or split)",
})

-- :MatBuild - Compile Rust backend binary
vim.api.nvim_create_user_command("MatBuild", function()
  require("mat").build()
end, {
  desc = "Compile Rust backend binary for mat.nvim",
})
