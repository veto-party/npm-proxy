import { useCallback, type FunctionComponent, type PropsWithChildren } from "react";
import { client } from "../../api";
import { useFastDelete } from "../../hook/useFastDelete";

export const TreeNode: FunctionComponent<PropsWithChildren & { packageName: string; packageNames: string[] }> = ({
    packageName,
    packageNames,
    children
}) => {


    const [details, onClick, deleted] = useFastDelete(useCallback(async () => {
        for (const pkgName of packageNames.filter((name) => name !== packageName )) {
            await client.delete(`/-/api/delete/${pkgName}`);
        }

        await client.delete(`/-/api/delete/${packageName}`);
    }, [ packageName, packageNames ]));

    if (deleted) {
        return null;
    }

    return (
        <div className="flex flex-col gap-y-2 bg-gray-700 rounded-2xl p-4">
            <div className="flex flex-row justify-between text-gray-200">
                <h1>{decodeURIComponent(packageName)}</h1>
                <button className="bg-amber-900 border-2 rounded-2xl px-6" type="button" onClick={onClick}>Delete</button>
            </div>
            {details}
            {children}
        </div>
    );
}
