import {OpenAI} from "openai"

const openai = new OpenAI({
    apiKey: import.meta.env.OPENAI_API_KEY,
    dangerouslyAllowBrowser: true,
    baseURL: location.origin + '/openai/v1'
});

const output = document.querySelector('#output')

async function main() {
    const stream = await openai.chat.completions.create({
        model: "gpt-3.5-turbo",
        messages: [{role: "user", content: "Say this is a test"}],
        stream: true,
    });

    for await (const chunk of stream) {
        output.textContent += JSON.stringify(chunk.choices[0]?.delta?.content) + "\n";
    }
}

main();

console.log('hello - there?')