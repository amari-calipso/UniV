class OddEvenMergeSort:
    def __init__(this, end):
        this.end = end

    def oddEvenMergeSort(this, array, length):
        p = 1
        while p < length:
            k = p
            while k > 0:
                j = k % p
                while j+k < length:
                    i = 0
                    while i < k:
                        if (i+j)//(p*2) == (i+j+k)//(p*2):
                            if i+j+k < length:
                                compSwap(array, i+j, i+j+k)
                        i += 1
                    j += k*2
                k //= 2
            p *= 2

    def compSwapCheck(this, array, a, b):
        if b < this.end:
            compSwap(array, a, b)

    def oddEvenMergeParallel(this, array, a, ia, im, ib, bLen, loop):
        t0 = Thread(
            this.oddEvenMergeParallel, [this, array, a, ia, (ia+im)//2, im, bLen, False])
        t1 = Thread(
            this.oddEvenMergeParallel, [this, array, a, im, (im+ib)//2, ib, bLen, False])
        if im-ia > 1:
            Thread_start(t0)
            Thread_start(t1)
        p = a+im*bLen*2
        for i in range(bLen):
            this.compSwapCheck(array, p+i-bLen, p+i)
        if im-ia > 1:
            Thread_join(t0)
            Thread_join(t1)
        if loop and bLen > 1:
            this.oddEvenMergeParallel(
                array, a, ia, im+im, ib+ib, bLen//2, True)

    def oddEvenMergeSortParallel(this, array, a, b):
        h = (b-a)//2
        if h > 1:
            t0 = Thread(
                this.oddEvenMergeSortParallel, [this, array, a, a+h])
            t1 = Thread(
                this.oddEvenMergeSortParallel, [this, array, a+h, b])
            Thread_start(t0)
            Thread_start(t1)
            Thread_join(t0)
            Thread_join(t1)
        for i in range(h):
            this.compSwapCheck(array, a+i, a+i+h)
        this.oddEvenMergeParallel(array, a, 0, 1, 2, (b-a)//4, True)


@Sort("Concurrent Sorts", "Odd Even Merge Sort", "Odd Even Merge")
def oddEvenMergeSortRun(array):
    OddEvenMergeSort(len(array)).oddEvenMergeSort(array, len(array))


@Sort("Concurrent Sorts", "Odd Even Merge Sort (Parallel)", "Odd Even Merge (Parallel)")
def oddEvenMergeSortRun(array):
    OddEvenMergeSort(len(array)).oddEvenMergeSortParallel(array, 0, 2**math.ceil(math.log2(len(array))))
