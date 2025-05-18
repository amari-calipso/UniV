class BottomUpMergeSort:
    def merge(this, c, d, lt, md, rt):
        i = lt
        j = md+1
        k = lt
        while i <= md and j <= rt:
            if c[i] <= c[j]:
                d[k].write(c[i])
                i += 1
            else:
                d[k].write(c[j])
                j += 1
            k += 1
        while i <= md:
            d[k].write(c[i])
            i += 1
            k += 1
        while j <= rt:
            d[k].write(c[j])
            j += 1
            k += 1

    def mergePass(this, x, y, s, n):
        i = 0
        while i <= n-2*s:
            this.merge(x, y, i, i+s-1, i+2*s-1)
            i += 2*s
        if i+s < n:
            this.merge(x, y, i, i+s-1, n-1)
        else:
            j = i
            while j <= n-1:
                y[j].write(x[j])
                j += 1

    def sort(this, array, n):
        if n < 16:
            binaryInsertionSort(array, 0, n)
            return
        s = 16
        speed = sortingVisualizer.getSpeed()
        sortingVisualizer.setSpeed(max(int(10*(len(array)/2048)), speed*4))
        i = 0
        while i <= n-16:
            binaryInsertionSort(array, i, i+16)
            i += 16
        binaryInsertionSort(array, i, n)
        sortingVisualizer.setSpeed(speed)
        b = sortingVisualizer.createValueArray(n)
        while s < n:
            this.mergePass(array, b, s, n)
            s *= 2
            this.mergePass(b, array, s, n)
            s *= 2


@Sort("Merge Sorts", "Bottom Up Merge Sort", "Bottom Up Merge")
def bottomUpMergeSortRun(array):
    BottomUpMergeSort().sort(array, len(array))
