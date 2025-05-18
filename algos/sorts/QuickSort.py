class QuickSort:
    def __init__(this, pSel):
        if pSel is None:
            this.pSel = UniV_getUserPivotSelection("Select pivot selection")
        else:
            this.pSel = sortingVisualizer.getPivotSelectionByName(pSel)

    def LRQuickSort(this, array, a, b):
        pSel = this.pSel
        while b-a > 1:
            array[a].swap(array[pSel(array, a, b)])
            p: int
            p = partition(array, a, b, a)
            array[a].swap(array[p])
            this.LRQuickSort(array, a, p)
            a = p+1

    def LRQuickSortParallel(this, array, a, b):
        pSel = this.pSel
        if b-a > 1:
            array[a].swap(array[pSel(array, a, b)])
            p: int
            p = partition(array, a, b, a)
            array[a].swap(array[p])
            t0 = Thread(
                this.LRQuickSortParallel, [this, array, a, p])
            t1 = Thread(
                this.LRQuickSortParallel, [this, array, p+1, b])
            Thread_start(t0)
            Thread_start(t1)
            Thread_join(t0)
            Thread_join(t1)

    def LLQuickSort(this, array, a, b):
        pSel = this.pSel
        while b-a > 1:
            array[b-1].swap(array[pSel(array, a, b)])
            p: int
            p = LLPartition(array, a, b)
            this.LLQuickSort(array, a, p)
            a = p+1

    def LLQuickSortParallel(this, array, a, b):
        pSel = this.pSel
        if b-a > 1:
            array[b-1].swap(array[pSel(array, a, b)])
            p: int
            p = LLPartition(array, a, b)
            t0 = Thread(
                this.LLQuickSortParallel, [this, array, a, p])
            t1 = Thread(
                this.LLQuickSortParallel, [this, array, p+1, b])
            Thread_start(t0)
            Thread_start(t1)
            Thread_join(t0)
            Thread_join(t1)


llQuickSortKillers = {
    "Linear": [
        "Reversed", "Reversed Sawtooth", "No shuffle", 
        "Sorted", "Few Random", "Noisy", "Scrambled Tail"
    ], 
    "Quadratic": [
        "Reversed", "Reversed Sawtooth", "Sawtooth", "No shuffle", 
        "Sorted", "Few Random", "Noisy", "Scrambled Tail"
    ],
    "Quintic": [
        "Reversed", "Reversed Sawtooth", "Sawtooth", "No shuffle", 
        "Sorted", "Few Random", "Noisy", "Scrambled Tail", "Random"
    ],
    "Sine Wave": [
        "Reversed", "Reversed Sawtooth", "Sawtooth", "No shuffle", 
        "Sorted", "Few Random", "Final Merge Pass", "Scrambled Head"
    ],
    "Perlin Noise": ["Sorted"]
}


lrQuickSortKillers = {
    "Linear":    ["Reversed Sawtooth", "Quicksort Adversary"], 
    "Quadratic": ["Reversed Sawtooth", "Sawtooth"], 
    "Quintic":   ["Reversed Sawtooth", "Sawtooth", "Quicksort Adversary"]
}


@Sort(
    "Quick Sorts", 
    "Quick Sort - Left/Right Pointers", 
    "LR Quick Sort",
    lrQuickSortKillers
)
def LRQuickSortRun(array):
    QuickSort(None).LRQuickSort(array, 0, len(array))


@Sort(
    "Quick Sorts", 
    "Quick Sort - Left/Left Pointers", 
    "LL Quick Sort",
    llQuickSortKillers
)
def LLQuickSortRun(array):
    QuickSort(None).LLQuickSort(array, 0, len(array))


@Sort(
    "Quick Sorts", 
    "Quick Sort - Left/Right Pointers (Parallel)", 
    "LR Quick Sort (Parallel)",
    lrQuickSortKillers
)
def LRQuickSortRun(array):
    QuickSort(None).LRQuickSortParallel(array, 0, len(array))


@Sort(
    "Quick Sorts", 
    "Quick Sort - Left/Left Pointers (Parallel)", 
    "LL Quick Sort (Parallel)",
    llQuickSortKillers
)
def LLQuickSortRun(array):
    QuickSort(None).LLQuickSortParallel(array, 0, len(array))
