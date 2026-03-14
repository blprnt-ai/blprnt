import { AnimatePresence, motion } from 'framer-motion'
import { XIcon } from 'lucide-react'
import { Button } from '@/components/atoms/button'
import { useAppViewModel } from '@/hooks/use-app-viewmodel'
import { cn } from '@/lib/utils/cn'

export const BlprntBanner = () => {
  const { bannerModel } = useAppViewModel()

  return (
    <AnimatePresence>
      {bannerModel.showBanner && (
        <div>
          <motion.div
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            initial={{ opacity: 1, y: -10 }}
            transition={{ duration: 0.3 }}
            className={cn(
              'w-full px-4 h-10 bg-black/60 text-primary border-b border-dashed flex items-center justify-between cursor-pointer hover:bg-black/20 transition-colors duration-300',
              bannerModel.type === 'warning' && 'border-warn text-warn',
            )}
            onClick={bannerModel.clickAction}
          >
            <div className="font-medium">{bannerModel.content}</div>
            <Button size="xs" variant="outline-ghost" onClick={bannerModel.dismiss}>
              <XIcon className="size-4" />
            </Button>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  )
}
