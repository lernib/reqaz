import createExpress from "express"
import * as fs from "fs/promises"
import * as path from "path"

const PORT = 5000
const CWD = process.cwd()

const app = createExpress()

async function path_exists(path: string): Promise<boolean> {
    try {
        await fs.stat(path)
        return true
    } catch {
        return false
    }
}

async function path_is_directory(path: string): Promise<boolean> {
    if (await path_exists(path)) {
        let stat = await fs.stat(path)

        return stat.isDirectory()
    }

    return false
}


app.get('*', async (req, res) => {
    let filePath = path.join(CWD, "src", req.path)

    if (await path_is_directory(filePath)) {
        filePath = path.join(filePath, "index.html")
    }

    if (!await path_exists(filePath)) {
        res.status(404).send("Not found")
    }

    res.sendFile(filePath)
})

app.listen(PORT, () => {
    console.log(`Running at http://localhost:${PORT}`)
})
