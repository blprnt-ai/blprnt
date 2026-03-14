import { debounce } from 'lodash'
import { AnimatePresence, motion } from 'motion/react'
import { Portal } from 'radix-ui'
import { useCallback, useEffect, useLayoutEffect, useMemo, useState } from 'react'
import ReactDOM from 'react-dom'
import { Button } from '@/components/atoms/button'
import { Confetti } from '@/components/atoms/confetti'
import { cn } from '@/lib/utils/cn'

interface TourStep {
  id: string
  allowNext?: boolean
  target: string
  inputTarget?: string
  title: string
  body: React.ReactNode
  proceedOnInput?: boolean
  skippable?: boolean
  placement?: 'right' | 'left' | 'top' | 'bottom' | 'center'
}

type TargetRect = Pick<DOMRect, 'bottom' | 'height' | 'left' | 'right' | 'top' | 'width'>

const defaultPadding = 8

const steps: TourStep[] = [
  {
    body: (
      <div className="flex flex-col gap-2">
        <div>Welcome to blprnt! Let's get started.</div>
        <div className="font-light text-muted-foreground">Create your first project.</div>
      </div>
    ),
    id: 'no-project-create-new-project',
    placement: 'bottom',
    target: '[data-tour="no-project-create-new-project"]',
    title: 'blprnt',
  },
  {
    body: (
      <div className="flex flex-col gap-2">
        <div>Give your project a name to help you identify it in the future.</div>
        <div className="font-light text-muted-foreground">
          You can have multiple projects to keep different ideas, code, or vibes separate.
        </div>
      </div>
    ),
    id: 'project-name',
    placement: 'bottom',
    target: '[data-tour="project-name"]',
    title: 'Create New Project',
  },
  {
    allowNext: false,
    body: (
      <div className="flex flex-col gap-2">
        <div>
          Click the 'Select Folder' button to add a folder to your project. This is "sandbox" for the AI to work in.
        </div>
        <div className="font-normal text-muted-foreground">
          The AI will only be able to access the files in this folder. Unless you
          <span className="rainbow"> YOLO </span>
          mode a session.
        </div>
      </div>
    ),
    id: 'working-directory-browse',
    placement: 'bottom',
    target: '[data-tour="working-directory-browse"]',
    title: 'Create New Project',
  },
  {
    body: "Click the 'Next' button to continue.",
    id: 'project-name-next',
    placement: 'left',
    target: '[data-tour="project-name-next"]',
    title: 'Create New Project',
  },
  {
    body: 'This is the agent primer for the project. It will be used to help the AI understand the project. It is completely optional and you can always add it later.',
    id: 'project-agent-primer-textarea',
    placement: 'left',
    skippable: true,
    target: '[data-tour="project-agent-primer-textarea"]',
    title: 'Create New Project',
  },
  {
    body: "Click the 'Create' button to create the project.",
    id: 'project-agent-primer-next',
    placement: 'left',
    target: '[data-tour="project-agent-primer-next"]',
    title: 'Create New Project',
  },
  {
    body: (
      <div className="flex flex-col gap-2">
        <div className="mb-2">Click the Settings tab to manage your models.</div>
        <div className="font-light text-muted-foreground">
          You need at lease one model enabled to create a valid session.
        </div>
      </div>
    ),
    id: 'user-account-models',
    placement: 'right',
    target: '[data-tour="user-account-models"]',
    title: 'Enable Your Models',
  },
  {
    body: (
      <div className="flex flex-col gap-2">
        <div>Click on a row to toggle the model's status.</div>
        <div className="font-light text-muted-foreground">
          We recommend enabling gpt-oss-120b and gpt-oss-20b to start. These are the free open-souce models from OpenAI.
        </div>
      </div>
    ),
    id: 'user-account-models-table',
    placement: 'left',
    target: '[data-tour="user-account-models-table"]',
    title: 'Enable Your Models',
  },
  {
    body: 'Click on your new project to expand it and see its contents.',
    id: 'sidebar-projects-expand-project',
    placement: 'right',
    target: '[data-tour="sidebar-project"]',
    title: 'Create New Session',
  },
  {
    body: 'Click on Create New Session to create a new session.',
    id: 'sidebar-create-new-session',
    placement: 'right',
    target: '[data-tour="sidebar-create-new-session"]',
    title: 'Create New Session',
  },
  {
    body: 'Give your session a name. You can always change it later.',
    id: 'session-name',
    inputTarget: '[data-tour="session-name-input"]',
    placement: 'bottom',
    target: '[data-tour="session-name"]',
    title: 'Create New Session',
  },
  {
    body: 'Choose a model for this session.',
    id: 'session-model-select',
    placement: 'right',
    target: '[data-tour="session-model-select"]',
    title: 'Create New Session',
  },
  {
    body: 'Click the "Create Session" button to create the session.',
    id: 'session-create-submit',
    placement: 'bottom',
    target: '[data-tour="session-create-submit"]',
    title: 'Create New Session',
  },
  {
    body: 'This is the main conversation area. Your entire histroy with the AI will be displayed here.',
    id: 'session-view',
    placement: 'left',
    target: '[data-tour="session-view"]',
    title: 'Session View',
  },
  {
    body: (
      <div className="flex flex-col gap-2">
        <div>Type your prompt here. Try it. It's free!</div>
        <div className="font-light text-muted-foreground">
          When the AI is working, a "Stop" button will appear. Just in case you need to nudge it in another direction.
        </div>
      </div>
    ),
    id: 'session-input',
    placement: 'top',
    proceedOnInput: false,
    target: '[data-tour="session-input"]',
    title: 'Session View',
  },
  {
    body: 'You can click this button to edit the session settings.',
    id: 'session-edit-session',
    placement: 'left',
    target: '[data-tour="session-edit-session"]',
    title: 'Session View',
  },
  {
    body: "That's it! Continue exploring the app. There are many more features to discover.",
    id: 'complete-tour',
    placement: 'center',
    target: '[data-tour="complete-tour"]',
    title: 'Welcome to blprnt!',
  },
]

const getTargetRect = (target: HTMLElement) => {
  const rect = target.getBoundingClientRect()
  return {
    bottom: rect.bottom,
    height: rect.height,
    left: rect.left,
    right: rect.right,
    top: rect.top,
    width: rect.width,
  }
}

const getPopoverPosition = (
  rect: TargetRect,
  placement: TourStep['placement'],
  popoverSize: { width: number; height: number },
) => {
  const gap = 16
  const viewportWidth = window.innerWidth
  const viewportHeight = window.innerHeight

  const placements: Record<NonNullable<TourStep['placement']>, { left: number; top: number }> = {
    bottom: {
      left: Math.min(Math.max(rect.left, gap), viewportWidth - popoverSize.width - gap),
      top: Math.min(rect.bottom + gap, viewportHeight - popoverSize.height - gap),
    },
    center: {
      left: viewportWidth / 2 - popoverSize.width / 2,
      top: viewportHeight / 2 - popoverSize.height / 2,
    },
    left: {
      left: Math.max(rect.left - popoverSize.width - gap, gap),
      top: Math.min(Math.max(rect.top, gap), viewportHeight - popoverSize.height - gap),
    },
    right: {
      left: Math.min(rect.right + gap, viewportWidth - popoverSize.width - gap),
      top: Math.min(Math.max(rect.top, gap), viewportHeight - popoverSize.height - gap),
    },
    top: {
      left: Math.min(Math.max(rect.left, gap), viewportWidth - popoverSize.width - gap),
      top: Math.max(rect.top - popoverSize.height - gap, gap),
    },
  }

  return placements[placement ?? 'right']
}

const useTourTarget = (step: TourStep) => {
  const [target, setTarget] = useState<HTMLElement | null>(null)

  useLayoutEffect(() => {
    const node = document.querySelector(step.target)
    setTarget(node as HTMLElement | null)
  }, [step])

  return target
}

const useTourInputTarget = (step: TourStep) => {
  const [inputTarget, setInputTarget] = useState<HTMLElement | null>(null)

  useLayoutEffect(() => {
    if (!step.inputTarget) return

    const node = document.querySelector(step.inputTarget)
    setInputTarget(node as HTMLElement | null)
  }, [step])

  return inputTarget
}

export const TourOverlay = ({ onComplete }: { onComplete: () => void }) => {
  const [showTour, setShowTour] = useState(true)
  const [stepIndex, setStepIndex] = useState(0)
  const step = steps[stepIndex]
  const target = useTourTarget(step)
  const inputTarget = useTourInputTarget(step)
  const [popoverSize, setPopoverSize] = useState({ height: 0, width: 0 })
  const [inputHasValue, setInputHasValue] = useState(false)
  const [windowSize, setWindowSize] = useState({ height: window.innerHeight, width: window.innerWidth })
  const [lastElementWasInput, setLastElementWasInput] = useState(false)

  useEffect(() => {
    const handleResize = debounce(() => {
      const width = window.innerWidth
      const height = window.innerHeight

      setWindowSize({ height, width })
    }, 10)

    window.addEventListener('resize', handleResize)
    return () => window.removeEventListener('resize', handleResize)
  }, [])

  // biome-ignore lint/correctness/useExhaustiveDependencies: depend on window size
  const rect = useMemo(() => {
    if (!target) return null
    return getTargetRect(target)
  }, [target, windowSize])

  const highlightStyle = useMemo(() => {
    if (!rect) return null
    return {
      height: rect.height + defaultPadding * 2,
      left: rect.left - defaultPadding,
      top: rect.top - defaultPadding,
      width: rect.width + defaultPadding * 2,
    }
  }, [rect])

  const popoverStyle = useMemo(() => {
    if (!rect) return null
    return getPopoverPosition(rect, step.placement, popoverSize)
  }, [rect, popoverSize, step.placement])

  const handleComplete = useCallback(() => {
    onComplete()
    setShowTour(false)
  }, [onComplete])

  const isTargetButton = useMemo(() => {
    if (!target) return false
    return (
      target instanceof HTMLButtonElement ||
      target.getAttribute('role') === 'button' ||
      target.tagName.toLowerCase() === 'a'
    )
  }, [target])

  const isTargetInput = useMemo(() => {
    if (!target) return false
    return (
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      target.getAttribute('contenteditable') === 'true' ||
      inputTarget instanceof HTMLInputElement ||
      inputTarget instanceof HTMLTextAreaElement ||
      inputTarget?.getAttribute('contenteditable') === 'true'
    )
  }, [target, inputTarget])

  const handleNext = useCallback(() => {
    setLastElementWasInput(isTargetInput)
    setTimeout(() => {
      if (stepIndex === steps.length - 1) handleComplete()
      else setStepIndex(stepIndex + 1)
    }, 200)
  }, [stepIndex, handleComplete, isTargetInput])

  const handleNextClick = useCallback(() => {
    // If target is a button, trigger its click (the click listener will auto-progress)
    if (isTargetButton && target) {
      target.click()
    } else {
      handleNext()
    }
  }, [isTargetButton, target, handleNext])

  // Focus input targets and watch for changes to auto-progress
  useEffect(() => {
    if (!target || !isTargetInput || step.proceedOnInput === false) {
      setInputHasValue(false)
      return
    }

    const inputTargetElem = inputTarget ? inputTarget : target

    // Check initial value
    const getInputValue = () =>
      inputTargetElem instanceof HTMLInputElement || inputTargetElem instanceof HTMLTextAreaElement
        ? inputTargetElem.value
        : inputTargetElem.textContent

    const initialValue = getInputValue()
    setInputHasValue(Boolean(initialValue && initialValue.length >= 1))

    // Focus the input element
    if (!lastElementWasInput) target.focus()

    const handleInput = () => {
      const value = getInputValue()
      const hasValue = Boolean(value && value.length >= 1)
      setInputHasValue(hasValue)

      if (hasValue) {
        handleNext()
      }
    }

    target.addEventListener('input', handleInput)
    return () => target.removeEventListener('input', handleInput)
  }, [target, inputTarget, isTargetInput, handleNext, lastElementWasInput, step.proceedOnInput])

  // Watch for button clicks to auto-progress
  useEffect(() => {
    if (!target) return

    const isButtonElement =
      target instanceof HTMLButtonElement ||
      target.getAttribute('role') === 'button' ||
      target.tagName.toLowerCase() === 'a'

    if (!isButtonElement) return

    const handleClick = () => {
      // Small delay to allow the button's action to complete first
      setTimeout(handleNext, 100)
    }

    target.addEventListener('click', handleClick)
    return () => target.removeEventListener('click', handleClick)
  }, [target, handleNext])

  // Create clip-path to cut out the target area from the backdrop
  const backdropClipPath = useMemo(() => {
    if (!highlightStyle) return undefined
    const { left, top, width, height } = highlightStyle
    const right = left + width
    const bottom = top + height
    // Polygon that traces the outer edge, then cuts out the inner rectangle
    return `polygon(
      0% 0%, 100% 0%, 100% 100%, 0% 100%, 0% 0%,
      ${left}px ${top}px, ${left}px ${bottom}px, ${right}px ${bottom}px, ${right}px ${top}px, ${left}px ${top}px
    )`
  }, [highlightStyle])

  if (!showTour || !step || !target || !rect || !highlightStyle || !popoverStyle) return null

  return ReactDOM.createPortal(
    <Portal.Root>
      {stepIndex === steps.length - 1 && <Confetti />}
      <AnimatePresence mode="popLayout">
        <motion.div
          key={stepIndex}
          animate={{ opacity: 1 }}
          className="pointer-events-none fixed inset-0 z-70"
          exit={{ opacity: 0 }}
          initial={{ opacity: 0 }}
        >
          <motion.div className="absolute inset-0 bg-black/40" style={{ clipPath: backdropClipPath }} />

          <motion.div
            animate={{ opacity: 1, scale: 1 }}
            className="pointer-events-none absolute rounded-lg border border-primary/50 bg-primary/10 shadow-[0_0_0_1px_rgba(59,130,246,0.4)]"
            exit={{ opacity: 0, scale: 0.98 }}
            initial={{ opacity: 0, scale: 0.98 }}
            style={highlightStyle}
            transition={{ damping: 30, stiffness: 260, type: 'spring' }}
          />

          <motion.div
            ref={(node) => {
              if (!node) return
              const { width, height } = node.getBoundingClientRect()
              if (width !== popoverSize.width || height !== popoverSize.height) {
                setPopoverSize({ height, width })
              }
            }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 8 }}
            initial={{ opacity: 0, y: 8 }}
            style={popoverStyle}
            transition={{ damping: 28, stiffness: 260, type: 'spring' }}
            className={cn(
              'pointer-events-auto absolute z-80 w-[340px] rounded-xl border border-yellow-500 border-dashed bg-background p-5 shadow-xl',
              'flex flex-col gap-3 text-foreground',
            )}
          >
            <div className="flex items-center justify-between">
              <div className="text-lg font-semibold">{step.title}</div>
              <Button className="text-muted-foreground/50" size="sm" variant="link" onClick={handleComplete}>
                Skip Tour
              </Button>
            </div>
            <div className="text-sm text-muted-foreground">{step.body}</div>

            <div className="flex items-center justify-between pt-2">
              <div className="text-xs text-muted-foreground">
                {stepIndex + 1} / {steps.length}
              </div>
              <div className="flex gap-2">
                {step.allowNext !== false && (
                  <Button
                    size="sm"
                    variant="default"
                    disabled={
                      isTargetInput && !inputHasValue && !(step.skippable === true) && step.proceedOnInput !== false
                    }
                    onClick={handleNextClick}
                  >
                    {stepIndex === steps.length - 1 ? 'Finish' : step.skippable === true ? 'Skip' : 'Next'}
                  </Button>
                )}
              </div>
            </div>
          </motion.div>
        </motion.div>
      </AnimatePresence>
    </Portal.Root>,
    document.body,
  )
}
