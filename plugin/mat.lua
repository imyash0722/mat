if vim.g.loaded_mat == 1 then
  return
end
vim.g.loaded_mat = 1

-- :Mat [file] or :'<,'>Mat
vim.api.nvim_create_user_command("Mat", function(args)
  if args.range == 2 then
    local lines = vim.api.nvim_buf_get_lines(0, args.line1 - 1, args.line2, false)
    require("mat").open_snippet(lines)
  else
    require("mat").open({ file = args.args })
  end
end, {
  nargs = "?",
  complete = "file",
  range = true,
  desc = "Open interactive Markdown preview in a floating modal",
})

-- :MatPreview
vim.api.nvim_create_user_command("MatPreview", function()
  require("mat").preview()
end, {
  desc = "Open live Markdown preview in a side split",
})

-- :MatPreviewToggle
vim.api.nvim_create_user_command("MatPreviewToggle", function()
  require("mat").preview_toggle()
end, {
  desc = "Toggle live Markdown preview split",
})

-- :MatClose
vim.api.nvim_create_user_command("MatClose", function()
  require("mat").close()
end, {
  desc = "Close active Markdown preview split",
})

-- :MatBuild
vim.api.nvim_create_user_command("MatBuild", function()
  require("mat").build()
end, {
  desc = "Compile Rust backend binary for mat.nvim",
})
