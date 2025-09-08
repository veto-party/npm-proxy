import { useEffect, type ComponentProps, type FunctionComponent, type PropsWithChildren } from "react";
import { ErrorBoundary, type ErrorBoundaryProps } from 'react-error-boundary';

const FallbackComponent  = (automatic: boolean) =>  ({
    resetErrorBoundary,
    error
}: ComponentProps<Exclude<ErrorBoundaryProps['FallbackComponent'], undefined>>) => {

    useEffect(() => console.error(error), [error]);

    useEffect(() => {
        if (automatic) {
            const timeout = setTimeout(() => resetErrorBoundary(), 5000);
            return () => clearTimeout(timeout);
        }
    }, [resetErrorBoundary]);

    return <div>
        <button type="button" onClick={resetErrorBoundary}>Retry</button>
    </div>
};

export const RetryErrorBoundary: FunctionComponent<PropsWithChildren & { automatic?: boolean; }> = ({
    children,
    automatic,
}) => {
    return <ErrorBoundary FallbackComponent={FallbackComponent(automatic ?? true)}>
        {children}
    </ErrorBoundary>
}