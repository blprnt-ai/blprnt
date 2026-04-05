export const restoreDoubleLineBreaks = (str: string) => str.replace(/\n/g, '\n\n')

export const restoreSingleLineBreaks = (str: string) => str.replace(/\n\n/g, '\n')
