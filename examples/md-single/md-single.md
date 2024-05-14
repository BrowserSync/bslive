```yaml bslive_input
servers:
  - bind_address: 0.0.0.0:5001
```

CSS path:

```yaml bslive_route
path: /app.css
```

```css
body {
    background: blue
}
```

HTML file:

```yaml bslive_route
path: /
```

```html

<body>
<p>Hello world?</p>
<link rel="stylesheet" href="/app.css"/>
<script type="module">
    const v = await fetch("http://127.0.0.1:5002/shane")
    const r = await v.json()
    console.log(r);
</script>
</body>
```