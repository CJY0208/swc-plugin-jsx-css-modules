# swc-plugin-jsx-css-modules

一个 SWC 插件，能够在 JSX 中无缝使用 CSS 模块，无需显式导入或属性。

## 功能

- 自动处理 JSX 中的 CSS 模块
- 无需显式的 `styles` 变量或自定义属性如 `styleName`
- 支持全局和局部类名
- 兼容各种 CSS 预处理器（CSS、SCSS、SASS、LESS）
- 与 SWC 一起具有高性能

## 安装

```bash
npm install --save-dev swc-plugin-jsx-css-modules
```

## 使用方法

1. 将插件添加到你的 `.swcrc` 文件中：

```json
{
  "jsc": {
    "experimental": {
      "plugins": [
        [
          "swc-plugin-jsx-css-modules",
          {
            "prefer": "local",
            "styleFileReg": ["\\.(css|scss|sass|less)$"]
          }
        ]
      ]
    }
  }
}
```

2. 在你的组件中使用 CSS 模块：

```jsx
import "./styles.css"; // 无需默认导入

const Component = () => (
  <div className="container">
    // 将被转换为使用 CSS 模块
    <span className="text">Hello</span>
  </div>
);
```

## 配置

- `prefer` (可选)：确定未指定类名是否应该被视为局部或全局。默认值："local"
- `styleFileReg` (可选)：用于匹配样式文件的正则表达式数组。默认值：["\.(css|scss|sass|less)$"]

## 特殊语法

你可以使用特殊语法显式地将类名标记为全局或局部：

```jsx
// 全局类名
<div className=":global(container)">...</div>

// 局部类名
<div className=":local(wrapper)">...</div>

// 混合
<div className=":global(container) :local(wrapper)">...</div>
```

## 开发

1. 克隆仓库
2. 安装依赖：

```bash
cargo build
```

3. 运行测试：

```bash
cargo test
```

## 许可

MIT

## 鸣谢

这是 [babel-plugin-jsx-css-modules](https://github.com/CJY0208/babel-plugin-jsx-css-modules) 的 SWC 版本
