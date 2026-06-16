import type { ComponentProps } from 'react'
import { CircleHelp } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Label } from '@/components/controls/label'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/controls/tooltip'

const FieldSet = ({ className, ...props }: ComponentProps<'fieldset'>) => (
  <fieldset className={cn('flex flex-col gap-6', className)} {...props} />
)

const Field = ({ className, ...props }: ComponentProps<'div'>) => (
  <div
    role="group"
    className={cn(
      'group/field data-[invalid=true]:text-destructive flex w-full gap-3',
      'flex-col [&>*]:w-full [&>.sr-only]:w-auto',
      className,
    )}
    {...props}
  />
)

const FieldContent = ({ className, ...props }: ComponentProps<'div'>) => (
  <div
    className={cn('group/field-content flex flex-1 flex-col gap-1.5 leading-snug', className)}
    {...props}
  />
)

type FieldLabelProps<TFieldValues extends Record<string, unknown> = Record<string, unknown>> = Omit<
  ComponentProps<typeof Label>,
  'htmlFor'
> & {
  htmlFor?: keyof TFieldValues & string
  hint?: string
}

const FieldLabel = <TFieldValues extends Record<string, unknown> = Record<string, unknown>>({
  className,
  hint,
  children,
  ...props
}: FieldLabelProps<TFieldValues>) => (
  <div className="flex w-fit items-center gap-1.5">
    <Label
      className={cn(
        'group/field-label peer/field-label flex gap-2 leading-snug group-data-[disabled=true]/field:opacity-50',
        className,
      )}
      {...props}
    >
      {children}
    </Label>
    {hint && (
      <Tooltip>
        <TooltipTrigger
          type="button"
          className="focus-visible:ring-ring/50 cursor-pointer rounded-full text-slate-500 outline-none hover:text-slate-300 focus-visible:text-slate-300 focus-visible:ring-[3px]"
          aria-label={
            typeof children === 'string' ? `More information about ${children}` : 'More information'
          }
        >
          <CircleHelp className="size-4" />
        </TooltipTrigger>
        <TooltipContent>{hint}</TooltipContent>
      </Tooltip>
    )}
  </div>
)

const FieldDescription = ({ className, ...props }: ComponentProps<'p'>) => (
  <p className={cn('text-sm leading-snug text-slate-400', className)} {...props} />
)

type FieldErrorProps = ComponentProps<'div'> & {
  errors?: Array<{ message?: string } | undefined> | undefined
}

const FieldError = ({ className, children, errors, ...props }: FieldErrorProps) => {
  if (!children && !errors?.length) return null

  let content = children

  if (!content) {
    const uniqueErrors = [...new Map(errors?.map((e) => [e?.message, e]) ?? []).values()]
    content =
      uniqueErrors.length === 1 ? (
        uniqueErrors[0]?.message
      ) : (
        <ul className="ml-4 flex list-disc flex-col gap-1">
          {uniqueErrors.map(
            (error, index) => error?.message && <li key={index}>{error.message}</li>,
          )}
        </ul>
      )
  }

  if (!content) return null

  return (
    <div role="alert" className={cn('text-destructive text-sm font-normal', className)} {...props}>
      {content}
    </div>
  )
}

export { Field, FieldLabel, FieldError, FieldSet, FieldContent, FieldDescription }
