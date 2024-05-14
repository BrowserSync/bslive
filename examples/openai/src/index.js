import {OpenAI} from "openai"

const openai = new OpenAI({
    apiKey: import.meta.env.OPENAI_API_KEY,
    dangerouslyAllowBrowser: true,
    baseURL: 'http://localhost:3000/openai/v1'
});

async function main() {
    const stream = await openai.chat.completions.create({
        model: "gpt-3.5-turbo",
        messages: [{role: "user", content: "Say this is a test"}],
        stream: true,
    });

    for await (const chunk of stream) {
        console.log(chunk.choices[0]?.delta?.content || "");
    }
}

main();