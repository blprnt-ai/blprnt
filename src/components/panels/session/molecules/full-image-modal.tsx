import { Dialog, DialogContent } from '@/components/atoms/dialog'

export const FullImageModal = ({ imageUrl, onClose }: { imageUrl: string; onClose: () => void }) => {
  return (
    <Dialog open={!!imageUrl} onOpenChange={onClose}>
      <DialogContent className="p-0" showCloseButton={false}>
        <img alt="Image" className="size-full object-cover" src={imageUrl} />
      </DialogContent>
    </Dialog>
  )
}
