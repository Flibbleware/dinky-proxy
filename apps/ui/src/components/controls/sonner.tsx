import {
  CircleCheckIcon,
  InfoIcon,
  Loader2Icon,
  OctagonXIcon,
  TriangleAlertIcon,
} from 'lucide-react'
import { Toaster as Sonner, type ToasterProps } from 'sonner'

// The app is hard-coded to a dark theme (see index.css), so the toaster matches directly
// rather than reading from a theme provider that was never mounted.
const Toaster = (props: ToasterProps) => (
  <Sonner
    theme="dark"
    position="top-right"
    className="toaster group"
    icons={{
      success: <CircleCheckIcon className="size-4" />,
      info: <InfoIcon className="size-4" />,
      warning: <TriangleAlertIcon className="size-4" />,
      error: <OctagonXIcon className="size-4" />,
      loading: <Loader2Icon className="size-4 animate-spin" />,
    }}
    {...props}
  />
)

export { Toaster }
