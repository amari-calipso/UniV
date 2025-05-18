class CombSort:
    def __init__(this, shrink):
        if shrink is None:
            tmp = sortingVisualizer.getUserInput(
                "Insert shrink factor:", "1.3", parseFloat)
            if tmp < 1:
                this.shrink = 1.3
            else:
                this.shrink = tmp
        else:
            this.shrink = shrink

    def sort(this, array, a, b):
        swapped = False
        gap = b-a
        while (gap > 1) or swapped:
            if gap > 1:
                gap /= this.shrink
                gap = int(gap)
            swapped = False
            i = a
            while gap+i < b:
                if array[i] > array[i+gap]:
                    array[i].swap(array[i+gap])
                    swapped = True
                i += 1


@Sort("Exchange Sorts", "Comb Sort", "Comb Sort")
def combSortRun(array):
    CombSort(None).sort(array, 0, len(array))
