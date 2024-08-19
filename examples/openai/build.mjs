import esbuild from "esbuild";

let ctx = await esbuild.build({
    entryPoints: ['./src/index.js'],
    bundle: true,
    outdir: 'dist',
    sourcemap: 'inline',
    metafile: true,
    plugins: [],
    format: 'esm',
    define: {
        'import.meta.env.OPENAI_API_KEY': JSON.stringify(process.env.OPENAI_API_KEY || '""')
    }
})

// console.log(esbuild.analyzeMetafileSync(ctx.metafile));