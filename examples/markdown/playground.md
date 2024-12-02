---
servers:
  - name: playground
    routes:
      - path: /
        dir: examples/basic/public
---

```html playground

<div class="main">
    Hello world!
</div>
```

```css 
@import url("reset.css");

:root {
    border: 1px dotted red;
}

* {
    font-family: system-ui
}
```

```js
console.log('Hello from playground.md')
```

