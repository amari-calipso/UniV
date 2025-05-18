class MergeInsertionSort:
    def blockSwap(this, array, a, b, s):
        while s > 0:
            s -= 1
            array[a].swap(array[b])
            a -= 1
            b -= 1

    def blockInsert(this, array, a, b, s):
        while a-s >= b:
            this.blockSwap(array, a-s, a, s)
            a -= s

    def blockReversal(this, array, a, b, s):
        b -= s
        while b > a:
            this.blockSwap(array, a, b, s)
            a += s
            b -= s

    def blockSearch(this, array, a, b, s, val):
        while a < b:
            m = a+(((b-a)//s)//2)*s
            if val < array[m]:
                b = m
            else:
                a = m+s
        return a

    def order(this, array, a, b, s):
        i = a
        j = i+s
        while j < b:
            this.blockInsert(array, j, i, s)
            i += s
            j += 2*s
        m = a+(((b-a)//s)//2)*s
        this.blockReversal(array, m, b, s)

    def sort(this, array, length):
        k = 1
        while 2*k <= length:
            i = 2*k-1
            while i < length:
                if array[i-k] > array[i]:
                    this.blockSwap(array, i-k, i, k)
                i += 2*k
            k *= 2
        while k > 0:
            a = k-1
            i = a+2*k
            g = 2
            p = 4
            while i+2*k*g-k <= length:
                this.order(array, i, i+2*k*g-k, k)
                b = a+k*(p-1)
                i += k*g-k
                j = i
                while j < i+k*g:
                    this.blockInsert(array, j, this.blockSearch(
                        array, a, b, k, array[j]), k)
                    j += k
                i += k*g+k
                g = p-g
                p *= 2
            while i < length:
                this.blockInsert(array, i, this.blockSearch(
                    array, a, i, k, array[i]), k)
                i += 2*k
            k //= 2


@Sort("Insertion Sorts", "Merge Insertion Sort [Ford-Johnson Algorithm]", "Merge Insert")
def mergeInsertionSortRun(array):
    MergeInsertionSort().sort(array, len(array))
