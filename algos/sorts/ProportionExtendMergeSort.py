class ProportionExtendMergeSort:
    MIN_INSERT = 8

    def partition(this, array, a, b, p):
        i = a-1
        j = b
        while True:
            i += 1
            while i < b and array[i] < array[p]:
                i += 1
            j -= 1
            while j >= a and array[j] > array[p]:
                j -= 1
            if i < j:
                array[i].swap(array[j])
            else:
                return i

    def mergeFW(this, array, a, m, b, p):
        pLen = m-a
        blockSwap(array, a, p, pLen)
        i = 0
        j = m
        k = a
        while i < pLen and j < b:
            if array[p+i] <= array[j]:
                array[k].swap(array[p+i])
                i += 1
            else:
                array[k].swap(array[j])
                j += 1
            k += 1
        while i < pLen:
            array[k].swap(array[p+i])
            i += 1
            k += 1

    def mergeBW(this, array, a, m, b, p):
        pLen = b-m
        blockSwap(array, m, p, pLen)
        i = pLen-1
        j = m-1
        k = b-1
        while i >= 0 and j >= a:
            if array[p+i] >= array[j]:
                array[k].swap(array[p+i])
                i -= 1
            else:
                array[k].swap(array[j])
                j -= 1
            k -= 1
        while i >= 0:
            array[k].swap(array[p+i])
            i -= 1
            k -= 1

    def smartMerge(this, array, a, m, b, p):
        if m-a < b-m:
            this.mergeFW(array, a, m, b, p)
        else:
            this.mergeBW(array, a, m, b, p)

    def mergeTo(this, array, a, m, b, p):
        i = a
        j = m
        while i < m and j < b:
            if array[i] <= array[j]:
                array[p].swap(array[i])
                i += 1
            else:
                array[p].swap(array[j])
                j += 1
            p += 1
        while i < m:
            array[p].swap(array[i])
            p += 1
            i += 1
        while j < b:
            array[p].swap(array[j])
            p += 1
            j += 1

    def pingPongMerge(this, array, a, m1, m, m2, b, p):
        p1 = p+m-a
        pEnd = p+b-a
        this.mergeTo(array, a, m1, m, p)
        this.mergeTo(array, m, m2, b, p1)
        this.mergeTo(array, p, p1, pEnd, a)

    def mergeSort(this, array, a, b, p):
        n = b-a
        j = n
        while (j+3)//4 >= this.MIN_INSERT:
            j = (j+3)//4
        i = a
        while i < b:
            binaryInsertionSort(array, i, min(b, i+j))
            i += j
        while j < n:
            i = a
            while i+2*j < b:
                this.pingPongMerge(array, i, i+j, i+2*j,
                                   min(i+3*j, b), min(i+4*j, b), p)
                i += 4*j
            if i+j < b:
                this.mergeBW(array, i, i+j, b, p)
            j *= 4

    def smartMergeSort(this, array, a, b, p, pb):
        if b-a <= pb-p:
            this.mergeSort(array, a, b, p)
            return
        m = (a+b)//2
        this.mergeSort(array, a, m, p)
        this.mergeSort(array, m, b, p)
        this.mergeFW(array, a, m, b, p)

    def sort(this, array, a, m, b):
        n = b-a
        if n < 4*this.MIN_INSERT:
            binaryInsertionSort(array, a, b)
            return
        if m-a <= n//3:
            t = (n+2)//3
            this.smartMergeSort(array, m, b-t, b-t, b)
            this.smartMerge(array, a, m, b-t, b-t)
            m = b-t
        m1 = (a+m)//2
        m2 = this.partition(array, m, b, m1)
        i = m
        j = m2
        while i > m1:
            i -= 1
            j -= 1
            array[i].swap(array[j])
        m = m2-(m-m1)
        if m-m1 < b-m2:
            this.mergeSort(array, m1, m, m2)
            this.smartMerge(array, a, m1, m, m2)
            this.sort(array, m+1, m2, b)
        else:
            this.mergeSort(array, m2, b, m1)
            this.smartMerge(array, m+1, m2, b, m1)
            this.sort(array, a, m1, m)


@Sort("Merge Sorts", "Proportion Extend Merge Sort", "Proportion Extend Merge")
def proportionExtendMergeSortRun(array):
    ProportionExtendMergeSort().sort(array, 0, 0, len(array))
