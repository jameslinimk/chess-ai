import { readFileSync, writeFileSync } from "fs"

// Taken from <https://www.chessgames.com/chessecohelp.html>
// Code's pretty messy, but it works
const raw_opens = readFileSync("raw_opens.txt", "utf8").split("\n")
const opens: {
    [name: string]: {
        code: string
        moves: string[][]
    }
} = {}

const notationToPos = (notation: string) => {
    const letters = "abcdefgh"
    const [letter, number] = notation
    return [letters.indexOf(letter), parseInt(number) - 1]
}

for (let i = 0; i < raw_opens.length; i++) {
    if (!raw_opens[i]) continue
    if (i % 2 == 0) {
        const moves: string[][] = []
        let inx = -1
        raw_opens[i + 1].split(" ").forEach((move) => {
            if (!isNaN(+move)) {
                moves[parseInt(move) - 1] = []
                inx++
                return
            }

            moves[inx].push(move)
        })
        const split = raw_opens[i].split("\t")
        const name = split[1]
        const code = split[0]

        opens[name] = {
            code,
            moves,
        }
    }
}

writeFileSync("../assets/openings.json", JSON.stringify(opens))
