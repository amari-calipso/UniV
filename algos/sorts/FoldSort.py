class FoldSort:
    def __init__(this, end):
        this.end = end

    def compSwap(this, array, a, b):
        if b < this.end and array[a] > array[b]:
            array[a].swap(array[b])

    def halver(this, array, a, b):
        while a < b:
            this.compSwap(array, a, b)
            a += 1
            b -= 1

    def sort(this, array, a, b):
        this.end = b
        ceilLog = 1
        while (1 << ceilLog) < (b-a):
            ceilLog += 1
        size = 1 << ceilLog
        k = size >> 1
        while k > 0:
            i = size
            while i >= k:
                j = a
                while j < b:
                    this.halver(array, j, j+i-1)
                    j += i
                i >>= 1
            k >>= 1

    def sortParallel(this, array, a, b, s, loop):
        h = (b-a)//2
        for i in range(h):
            this.compSwap(array, a+i, b-1-i)
        if h >= s//2:
            t0 = Thread(
                this.sortParallel, [this, array, a, a+h, s, False])
            t1 = Thread(
                this.sortParallel, [this, array, a+h, b, s, False])
            Thread_start(t0)
            Thread_start(t1)
            Thread_join(t0)
            Thread_join(t1)
        if loop and s > 2:
            this.sortParallel(array, a, b, s//2, True)


@Sort("Concurrent Sorts", "Fold Sorting Network", "Fold Sort")
def foldSortRun(array):
    FoldSort(len(array)).sort(array, 0, len(array))


@Sort("Concurrent Sorts", "Fold Sorting Network (Parallel)", "Fold Sort (Parallel)")
def foldSortRun(array):
    FoldSort(len(array)).sortParallel(array, 0, 2**math.ceil(
        math.log2(len(array))), 2**math.ceil(math.log2(len(array))), True)
