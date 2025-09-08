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

        const counters: Record<string, number> = {};
        const results: Record<string, string[]> = {};

        for (const pkgName of packages) {

            results[pkgName] ??= [];

            for (const subPkgName of packages) {
                if (!subPkgName.startsWith(pkgName)) {
                    continue;
                }

                const sub = subPkgName.substring(pkgName.length);
                if (!sub.startsWith('/') && !sub.startsWith(encodeURI('/'))) {
                    continue;
                }

                results[pkgName].push(subPkgName);
                counters[subPkgName]++;
            }
        }

        for (const key in counters) {
            delete results[key];
        }

        return Object.entries(results);
    }, [ packages ]);

    return <div className="flex flex-col gap-y-6">
        <input value={search} onChange={e => setSearch(e.target.value)} type="text" placeholder="filter packages..." />
        {packageGroups.sort(([a], [b]) => a.localeCompare(b)).map(([pgkName, metadatas]) => (
            <TreeNode packageName={pgkName} packageNames={metadatas}>
                {metadatas.sort().map((packageName) => <ConfirmDeleteButton key={packageName} packageName={packageName}/>)}
            </TreeNode>
        ))}
    </div>
}