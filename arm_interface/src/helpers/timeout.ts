export const timeout = (duration: number): Promise<void> => {
    return new Promise<void>((resolve) => {
        setTimeout(resolve, duration);
    });
}