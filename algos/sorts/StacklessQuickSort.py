class StacklessQuickSort:
    def __init__(this, pSel):
        if pSel is None:
            this.pSel = UniV_getUserPivotSelection("Select pivot selection")
        else:
            this.pSel = sortingVisualizer.getPivotSelectionByName(pSel)

    def partition(this, array, a, b):
        pSel = this.pSel
        i = a
        j = b
        array[a].swap(array[pSel(array, a, b)])
        while True:
            while True:
                i += 1
                if not (i < j and array[i] < array[a]):
                    break
            while True:
                j -= 1
                if not (j >= i and array[j] >= array[a]):
                    break
            if i < j:
                array[i].swap(array[j])
            else:
                array[a].swap(array[j])
                return j

    def sort(this, array, a, b):
        pSel = this.pSel
        max_ = findMaxValue(array, a, b)
        i = b-1
        while i >= 0:
            if array[i] == max_:
                b -= 1
                array[i].swap(array[b])
            i -= 1
        b1 = b
        med = True
        while True:
            while b1-a > 16:
                if med:
                    array[a].swap(array[pSel(array, a, b1)])
                p = this.partition(array, a, b1)
                array[p].swap(array[b])
                b1 = p
            binaryInsertionSort(array, a, b1)
            a = b1+1
            if a >= b:
                if a-1 < b:
                    array[a-1].swap(array[b])
                return
            b1 = lrBinarySearch(array, a, b, array[a-1], True)
            array[a-1].swap(array[b])
            med = True
            while a < b1 and array[a-1] == array[a]:
                med = False
                a += 1
            if a == b1:
                med = True


@Sort(
    "Quick Sorts", 
    "Stackless Quick Sort", 
    "Stackless Quick",
    {
        "Linear":    ["Quicksort Adversary"], 
        "Quadratic": ["Reversed Sawtooth", "Sawtooth"], 
        "Quintic":   ["Reversed Sawtooth", "Sawtooth", "Quicksort Adversary"]
    }
)
def stacklessQuickSortRun(array):
    StacklessQuickSort(None).sort(array, 0, len(array))
