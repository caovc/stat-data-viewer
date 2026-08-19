export async function consumePendingOpenPaths(
  take: () => Promise<string[]>,
  openPath: (path: string) => Promise<unknown>,
) {
  const paths = await take()
  for (const path of paths) await openPath(path)
  return paths
}
