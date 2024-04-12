import chalk from "chalk"


export class ErrorTypeError extends Error {
    private err: Error;

    constructor(e: Error) {
        super(e.message)

        this.err = e;
    }
}

export function errorTypeErrorEncapsulate(e: Error): ErrorTypeError {
    return new ErrorTypeError(e)
}

function colorStatus(status: number): string {
    if (status < 200) {
        return chalk.blue.bold(status)
    } else if (status < 300) {
        return chalk.green.bold(status)
    } else if (status < 400) {
        return chalk.yellow.bold(status)
    } else if (status < 500) {
        return chalk.red.bold(status)
    } else {
        return chalk.magentaBright.bold(status)
    }
}

export function logSourceRequest(status: number, locator: string) {
    const statusStr = colorStatus(status)

    console.info(`[${statusStr}] ${locator}`)
}
