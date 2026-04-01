export const formatDate = (value: Date) => {
  if (Number.isNaN(value.getTime())) return 'Unknown'

  return new Intl.DateTimeFormat('en', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(value)
}

export const formatDirectoryCount = (count: number) => {
  return `${count} ${count === 1 ? 'directory' : 'directories'}`
}
