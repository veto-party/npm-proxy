import type { FunctionComponent, PropsWithChildren } from "react";

export const TreeNode: FunctionComponent<PropsWithChildren & { packageName: string; }> = ({
    packageName,
    children
}) => {
    return (
        <div>
            <h1>{decodeURIComponent(packageName)}</h1>
            <div>{children}</div>
        </div>
    );
}