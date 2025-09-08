import { useCallback, useMemo, useState, type FunctionComponent } from "react";
import { client } from "../api";
import { ConfirmDeleteButton } from "./Tree/ConfirmDeleteButton";
import { useLoaded } from "../hook/useLoaded";
import { TreeNode } from "./Tree/TreeNode";

const fuzzyMatch = (pattern: string): RegExp => {
    return new RegExp(pattern.split("").map(s => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join(".*"), "i");
}

export const AllPackagesLoader: FunctionComponent = () => {
    
    const [search, setSearch] = useState<string>("");

    const rawPackages = useLoaded(useCallback(async (): Promise<string[]> => {
        try {
            return await client.get("/-/api/all").then((result) => result.data);
        } catch (error) {
            throw new Error("Cannot load api.", {
                cause: error
            });
        }
    }, []));

    const regex = useMemo(() => fuzzyMatch(encodeURIComponent(search)), [search]);

    const packages = useMemo(() => {
        if (search.trim() === '') {
            return rawPackages;
        }

        return rawPackages?.filter(regex.test.bind(regex));
    }, [search, regex, rawPackages])

    const packageGroups = useMemo(() => {
        if (!packages) {
            return [];
        }

        const results: Record<string, string[]> = {};
        for (const pkgName of packages) {
            for (const subPkgName of packages) {
                if (subPkgName.startsWith(pkgName)) {
                    results[pkgName] ??= [];
                    results[pkgName].push(subPkgName);
                }
            }
        }

        return Object.entries(results);
    }, [ packages ]);

    return <div className="flex flex-col gap-y-6">
        <input value={search} onChange={e => setSearch(e.target.value)} type="text" placeholder="filter packages..." />
        {packageGroups.map(([pgkName, metadatas]) => (
            <TreeNode packageName={pgkName}>
                {metadatas.map((packageName) => <ConfirmDeleteButton key={packageName} packageName={packageName}/>)}
            </TreeNode>
        ))}
    </div>
}