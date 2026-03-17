import { useEffect } from 'react'
import { reportError } from '@/lib/utils/error-reporting'

// @ts-expect-error
const AudioContext = globalThis.AudioContext || globalThis.webkitAudioContext
const audioContext = new AudioContext()

const sources = new Map<string, AudioBufferSourceNode | null>()

export const useAudio = (filepath: string) => {
  useEffect(() => {
    try {
      if (sources.has(filepath)) {
        return () => {
          sources.delete(filepath)
        }
      }

      sources.set(filepath, null)

      const source = audioContext.createBufferSource()
      sources.set(filepath, source)

      getFile(filepath).then((audioBuffer) => {
        source.buffer = audioBuffer
        source.connect(audioContext.destination)
      })
    } catch (error) {
      reportError(error, 'creating audio source')
    }

    return () => {
      sources.delete(filepath)
    }
  }, [filepath])

  return {
    play: async () => {
      if (!sources.has(filepath)) return

      try {
        sources.get(filepath)?.start()
        sources.set(filepath, null)
      } catch (error) {
        reportError(error, 'playing audio')
      }
    },
  }
}

const getFile = async (filepath: string) => {
  const fullFilePath = `${location.protocol}//${location.host}/${filepath}`
  const response = await fetch(fullFilePath)
  const arrayBuffer = await response.arrayBuffer()
  const audioBuffer = await audioContext.decodeAudioData(arrayBuffer)

  return audioBuffer
}
