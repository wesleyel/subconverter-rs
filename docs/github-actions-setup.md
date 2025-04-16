# GitHub Actions 工作流设置指南

本文档介绍如何设置GitHub Actions来自动构建和发布`subconverter-wasm`包到npm，以及更新Vercel项目。

## 工作流概述

本仓库包含两个主要工作流：

1. **publish-wasm.yml**: 构建并发布`subconverter-wasm`包到npm
2. **update-vercel.yml**: 在WASM包发布后更新Vercel项目

## 设置NPM_TOKEN

为了让GitHub Actions能够发布到npm，您需要设置一个NPM访问令牌：

1. 登录到npm网站(https://www.npmjs.com/)
2. 点击右上角的头像，选择"Access Tokens"
3. 点击"Generate New Token"
4. 选择"Automation"类型
5. 复制生成的令牌

然后，将此令牌添加到GitHub仓库的Secrets中：

1. 在GitHub仓库页面，点击"Settings"
2. 在左侧菜单中选择"Secrets and variables" > "Actions"
3. 点击"New repository secret"
4. 名称填写"NPM_TOKEN"，值填写您刚刚复制的npm令牌
5. 点击"Add secret"

## 触发工作流

### 发布WASM包

`publish-wasm.yml`工作流可以通过以下方式触发：

1. **手动触发**：在仓库的"Actions"选项卡中选择"Build and Publish subconverter-wasm"工作流，点击"Run workflow"

2. **推送到主分支**：当有更改推送到`main`或`master`分支，并且更改包含`wasm/**`目录中的文件时

3. **创建发布**：当在GitHub上创建新的Release时

### 更新Vercel项目

`update-vercel.yml`工作流会在WASM包成功发布后自动触发，也可以手动触发。

## 版本控制

- 当通过**推送**触发时，会发布预发布版本，使用`beta`标签
- 当通过**创建发布**触发时，会发布正式版本

## 持续集成

两个工作流协同工作，实现以下流程：

1. 修改Rust代码
2. 推送到仓库或创建发布
3. 自动构建WASM并发布到npm
4. 自动更新Vercel项目使用最新的WASM包
5. 创建PR以提交更改

## 注意事项

- 确保仓库有正确的目录结构
- 发布到npm需要有包的所有权或适当的权限
- 请勿将NPM_TOKEN泄露或公开
- 您可能需要初始创建和发布`subconverter-wasm`包

## 手动部署

如果自动工作流失败，您可以按照以下步骤手动构建和发布：

```bash
# 构建WASM
wasm-pack build --target web --out-dir pkg

# 进入pkg目录
cd pkg

# 编辑package.json设置版本号

# 发布到npm
npm publish
```

## 故障排除

如果遇到问题，请检查：

1. GitHub Actions日志中的错误信息
2. 确保NPM_TOKEN设置正确且未过期
3. 确保package.json中的名称未被占用
4. 检查是否有必要的权限来创建PR 