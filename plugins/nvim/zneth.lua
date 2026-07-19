local z_lang_group = vim.api.nvim_create_augroup("ZLanguageSetup", { clear = true })
vim.api.nvim_create_autocmd({ "BufRead", "BufNewFile" }, {
  pattern = "*.z",
  group = z_lang_group,
  callback = function()
    vim.bo.filetype = "zneth"
    vim.cmd("syntax on")
    vim.cmd([[
      syntax keyword mylangKeyword if else while ret fn constptr printf readf signed unsigned struct enum unsafe module void static i32 i64 i128 f32 f64 str i8 i1 !include
      syntax keyword mylangBoolean true false

      syntax match znethfunction "\v<\w+>(\s*\()@="
      syntax match znethid "\v(signed|unsigned)\s+\zs<\w+>"

      syntax match mylangNumber "\v<\d+>"
      syntax match mylangComment "\v//.*$"
      syntax region mylangString start=/"/ skip=/\\"/ end=/"/
    ]])
    vim.api.nvim_set_hl(0, 'mylangKeyword', { link = "Keyword", bold = true })
    vim.api.nvim_set_hl(0, 'mylangBoolean', { link = "Boolean", bold = true })
    vim.api.nvim_set_hl(0, 'mylangComment', { link = "Comment", italic = true })
    vim.api.nvim_set_hl(0, 'mylangString', { link = "String" })
    vim.api.nvim_set_hl(0, 'mylangNumber', { link = "Number" })
    vim.api.nvim_set_hl(0, 'znethfunction', { link = "Function" })
    vim.api.nvim_set_hl(0, 'znethid', { link = "Identifier" })
    vim.b.current_syntax = "zneth"
  end,
})
local vz_lang_group = vim.api.nvim_create_augroup("VDOLanguageSetup", { clear = true })
vim.api.nvim_create_autocmd({ "BufRead", "BufNewFile" }, {
  pattern = "*.vdo",
  group = vz_lang_group,
  callback = function()
    vim.bo.filetype = "vdo"
    vim.cmd("syntax on")
    vim.cmd([[
      syntax keyword mylangKeyword package
      syntax keyword mylangBoolean true false

      syntax match mylangNumber "\v<\d+>"
      syntax match mylangComment "\v//.*$"
      syntax region mylangString start=/"/ skip=/\\"/ end=/"/
    ]])
    vim.api.nvim_set_hl(0, 'mylangKeyword', { link = "Keyword", bold = true })
    vim.api.nvim_set_hl(0, 'mylangBoolean', { link = "Boolean", bold = true })
    vim.api.nvim_set_hl(0, 'mylangComment', { link = "Comment", italic = true })
    vim.api.nvim_set_hl(0, 'mylangString', { link = "String" })
    vim.api.nvim_set_hl(0, 'mylangNumber', { link = "Number" })
    vim.b.current_syntax = "vdo"
  end,
})
