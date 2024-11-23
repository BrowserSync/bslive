```html playground

<div class="main">
    Hello world 6.0
</div>
```

```css 
@import url("reset.css");

:root {
    border: 50px dotted orange;
}

* {
    font-family: system-ui
}
```

```js
let int = 0;
setInterval(() => {
    int += 1;
    document.body.textContent += int
}, 1000);
```

