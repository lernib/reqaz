import createExpress from "express"
import { resolveSource } from "./source.js"


const PORT = 5000
const app = createExpress()

app.get('*', async (req, res) => {
    let url = new URL(`${req.protocol}://${req.get('host')}${req.originalUrl}`)
    let source = await resolveSource(url);

    res.status(source.status)
        .send(source.body)
})

app.listen(PORT, () => {
    console.log(`Running at http://localhost:${PORT}`)
})
