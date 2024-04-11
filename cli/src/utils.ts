
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
