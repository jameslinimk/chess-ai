import { readFileSync, writeFileSync } from "fs"

// Taken from <https://www.chessgames.com/chessecohelp.html>
// Code's pretty messy, but it works
const raw_opens = readFileSync("raw_opens.txt", "utf8").replaceAll("\r\n", "\n").split("\n")
const opens: {
    name: string
    code: string
    moves: string[]
}[] = []

for (let i = 0; i < raw_opens.length; i++) {
    if (!raw_opens[i]) continue
    if (i % 2 == 0) {
        let moves: string[] = []
        raw_opens[i + 1].split(" ").forEach((move) => {
            if (!isNaN(+move)) return
            moves.push(move)
        })
        moves = moves.filter((v) => v.length)

        const split = raw_opens[i].split("\t")
        const name = split[1]
        const code = split[0]

        opens.push({
            name,
            code,
            moves,
        })
    }
}

writeFileSync("openings.json", JSON.stringify(opens))
