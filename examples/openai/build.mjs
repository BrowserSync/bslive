import esbuild from "esbuild";

let ctx = await esbuild.context({
    entryPoints: ['./src/index.js'],
    bundle: true,
    outdir: 'dist',
    sourcemap: 'inline',
    plugins: [],
    format: 'esm',
    define: {
        'import.meta.env.OPENAI_API_KEY': JSON.stringify(process.env.OPENAI_API_KEY)
    }
})

await ctx.watch()

let {host, port} = await ctx.serve({
    servedir: '.',
})

console.log(`http://${host}:${port}`)
