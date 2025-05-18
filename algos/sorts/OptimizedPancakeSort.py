class OptimizedPancakeSort:
    def flip(this, array, n):
        reverse(array, 0, n)

    def merge(this, array, h1, h2) -> bool:
        if h1 == 1 and h2 == 1:
            if array[0] > array[1]:
                this.flip(array, 2)
            return True
        n = h1+h2
        m = n//2
        if h2 < h1:
            if h2 < 1:
                return True
            i = 0
            j = h2
            while i < j:
                k = (i+j)//2
                if array[n-1-k-m] > array[n-1-k]:
                    i = k+1
                else:
                    j = k
            this.flip(array, n-m-i)
            this.flip(array, n-i)
            if this.merge(array, h2-i, i+m-h2):
                this.flip(array, m)
            this.flip(array, n)
            if not this.merge(array, i, n-m-i):
                this.flip(array, n-m)
        else:
            if h1 < 1:
                return False
            i = 0
            j = h1
            while i < j:
                k = (i+j)//2
                if array[k] < array[k+m]:
                    i = k+1
                else:
                    j = k
            this.flip(array, i)
            this.flip(array, i+m)
            if this.merge(array, i+m-h1, h1-i):
                this.flip(array, m)
            this.flip(array, n)
            if not this.merge(array, n-m-i, i):
                this.flip(array, n-m)
        return True

    def sort(this, array, n):
        if n < 2:
            return
        h = n//2
        this.sort(array, h)
        this.flip(array, n)
        this.sort(array, n-h)
        this.merge(array, n-h, h)


@Sort("Pancake Sorts", "Optimized Pancake Sort", "Optimized Pancake")
def optimizedPancakeSortRun(array):
    OptimizedPancakeSort().sort(array, len(array))
