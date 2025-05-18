class CombSort3Smooth:
    def powOfThree(this, array, a, b, g):
        if 3*g < b-a:
            this.powOfThree(array, a, b, 3*g)
            this.powOfThree(array, a+g, b, 3*g)
            this.powOfThree(array, a+g+g, b, 3*g)
        for i in range(a+g, b, g):
            compSwap(array, i-g, i)

    def combSort(this, array, a, b, g):
        if 2*g < b-a:
            this.combSort(array, a, b, 2*g)
            this.combSort(array, a+g, b, 2*g)
        this.powOfThree(array, a, b, g)

    def powOfThreeParallel(this, array, a, b, g):
        if 3*g < b-a:
            t0 = Thread(
                this.powOfThreeParallel, [this, array, a, b, 3*g])
            t1 = Thread(
                this.powOfThreeParallel, [this, array, a+g, b, 3*g])
            t2 = Thread(
                this.powOfThreeParallel, [this, array, a+g+g, b, 3*g])
            Thread_start(t0)
            Thread_start(t1)
            Thread_start(t2)
            Thread_join(t0)
            Thread_join(t1)
            Thread_join(t2)
        for i in range(a+g, b, g):
            compSwap(array, i-g, i)

    def combSortParallel(this, array, a, b, g):
        if 2*g < b-a:
            t0 = Thread(
                this.combSortParallel, [this, array, a, b, 2*g])
            t1 = Thread(
                this.combSortParallel, [this, array, a+g, b, 2*g])
            Thread_start(t0)
            Thread_start(t1)
            Thread_join(t0)
            Thread_join(t1)
        this.powOfThreeParallel(array, a, b, g)


@Sort("Concurrent Sorts", "Combsort with 3-Smooth Gaps", "3-Smooth Comb")
def combSort3SmoothRun(array):
    CombSort3Smooth().combSort(array, 0, len(array), 1)


@Sort("Concurrent Sorts", "3-Smooth Combsort (Parallel)", "3-Smooth Comb (Parallel)")
def weaveSortRun(array):
    CombSort3Smooth().combSortParallel(array, 0, len(array), 1)
