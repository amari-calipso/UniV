class BitonicSort:
    def greaterPowerOfTwoLessThan(this, n):
        k = 1
        while k < n:
            k <<= 1
        return k >> 1

    def compare(this, array, a, b, dir):
        if dir:
            if array[a] > array[b]:
                array[a].swap(array[b])
        else:
            if array[a] < array[b]:
                array[a].swap(array[b])

    def merge(this, array, a, l, dir):
        if l > 1:
            m = this.greaterPowerOfTwoLessThan(l)
            i = a
            while i < a+l-m:
                this.compare(array, i, i+m, dir)
                i += 1
            this.merge(array, a, m, dir)
            this.merge(array, a+m, l-m, dir)

    def mergeParallel(this, array, a, l, dir):
        if l > 1:
            m = this.greaterPowerOfTwoLessThan(l)
            i = a
            while i < a+l-m:
                this.compare(array, i, i+m, dir)
                i += 1
            t0 = Thread(
                this.mergeParallel, [this, array, a, m, dir])
            t1 = Thread(
                this.mergeParallel, [this, array, a+m, l-m, dir])
            Thread_start(t0)
            Thread_start(t1)
            Thread_join(t0)
            Thread_join(t1)

    def sort(this, array, a, l, dir):
        if l > 1:
            m = l//2
            this.sort(array, a, m, not dir)
            this.sort(array, a+m, l-m, dir)
            this.merge(array, a, l, dir)

    def sortParallel(this, array, a, l, dir):
        if l > 1:
            m = l//2
            t0 = Thread(
                this.sortParallel, [this, array, a, m, not dir])
            t1 = Thread(
                this.sortParallel, [this, array, a+m, l-m, dir])
            Thread_start(t0)
            Thread_start(t1)
            Thread_join(t0)
            Thread_join(t1)
            this.mergeParallel(array, a, l, dir)


@Sort("Concurrent Sorts", "Bitonic Sort", "Bitonic Sort")
def bitonicSortRun(array):
    BitonicSort().sort(array, 0, len(array), True)


@Sort("Concurrent Sorts", "Bitonic Sort (Parallel)", "Bitonic Sort (Parallel)")
def bitonicSortRun(array):
    BitonicSort().sortParallel(array, 0, len(array), True)
