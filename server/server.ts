import express from "express"
import { resolve } from "path"

const port = 3252
const app = express()

app.use("/docs", express.static(resolve("../target/doc/chess_ai")))
app.use(express.static(resolve("../target/doc")))

app.use(express.static(resolve("./static")))
app.use("/assets", express.static(resolve("../assets")))
app.get("/game.wasm", (_, res) => {
    res.sendFile(resolve("../target/wasm32-unknown-unknown/release/chess-ai.wasm"))
})

app.get("/", (_, res) => {
    res.sendFile(resolve("./index.html"))
})

app.listen(port, () => {
    console.log(`Listening at http://localhost:${port}`)
})
