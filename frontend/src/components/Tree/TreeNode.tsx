import { useCallback, type FunctionComponent, type PropsWithChildren } from "react";
import { client, packageClient } from "../../api";
import { useFastDelete } from "../../hook/useFastDelete";

export const TreeNode: FunctionComponent<PropsWithChildren & { packageName: string; packageNames: string[]; all: string[]; }> = ({
    packageName,
    packageNames,
    all,
    children
}) => {


    const [details, onClick, deleted] = useFastDelete(useCallback(async () => {
        for (const pkgName of packageNames.filter((name) => name !== packageName )) {
            await client.delete(`/-/api/delete/${pkgName}`);
        }

        await client.delete(`/-/api/delete/${packageName}`);
    }, [ packageName, packageNames ]));

    const [deleteAllDetails, onClickDeleteAll] = useFastDelete(useCallback(async () => {
        const packages = new Set<string>();
        const lookup = [packageName];

        while (lookup.length > 0) {
            const name = lookup.pop()!;
            if (packages.has(name)) {
                continue;
            }

            if (!all.includes(name)) {
                continue;
            }

            packages.add(name);
            const response = (await packageClient.get(name)).data;
            for (const version of Object.values(response.versions ?? {}) as any[]) {
                lookup.push(
                    ...Object.keys(version.dependencies  ?? {}).map(encodeURIComponent),
                    ...Object.keys(version.optionalDependencies ?? {}).map(encodeURIComponent),
                );
            }
        }

        for (const pkgName of Array.from(packages).reverse()) {
            await client.delete(`/-/api/delete/${pkgName}`);
        }
    }, [ packageName ]));
    

    if (deleted) {
        return null;
    }

    return (
        <div className="flex flex-col gap-y-2 bg-gray-700 rounded-2xl p-4">
            <div className="flex flex-row justify-between text-gray-200">
                <h1>{decodeURIComponent(packageName)}</h1>
                <div className="flex flex-row justify-end gap-x-4">
                    <button className="bg-amber-900 border-2 rounded-2xl px-6" type="button" onClick={onClick}>Delete</button>
                    <button className="bg-amber-900 border-2 rounded-2xl px-6" type="button" onClick={onClickDeleteAll}>Delete all</button>
                </div>
            </div>
            {deleteAllDetails}
            {details}
            {children}
        </div>
    );
}
