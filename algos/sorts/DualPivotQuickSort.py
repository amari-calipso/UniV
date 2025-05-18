def dualPivotQuickSort(array, a, b):
    while b-a > 32:
        m = a+((b-a)//2)
        compSwap(array, m, m+1)
        array[a].swap(array[m])
        array[b].swap(array[m+1])
        p1: int
        p2: int
        p1, p2 = dualPivotPartition(array, a, b)
        dualPivotQuickSort(array, a, p1)
        dualPivotQuickSort(array, p1, p2)
        a = p2
    uncheckedInsertionSort(array, a, b+1)


dualPivotQuickSortKillers = {
    "Linear":    ["Quicksort Adversary"], 
    "Quadratic": ["Reversed Sawtooth", "Sawtooth"], 
    "Quintic":   ["Reversed Sawtooth", "Sawtooth", "Quicksort Adversary"]
}


@Sort(
    "Quick Sorts", 
    "Dual Pivot QuickSort", 
    "Dual Pivot Quick",
    dualPivotQuickSortKillers
)
def dualPivotQuickSortRun(array):
    dualPivotQuickSort(array, 0, len(array)-1)
