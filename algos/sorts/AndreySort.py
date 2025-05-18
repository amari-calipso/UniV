class AndreySort:
    def sort(this, array, a, b):
        while b > 1:
            k = 0
            i = 1
            while i < b:
                if array[a+k] > array[a+i]:
                    k = i
                i += 1
            array[a].swap(array[a+k])
            a += 1
            b -= 1

    def backmerge(this, array, a1, l1, a2, l2):
        a0 = a2+l1
        while True:
            if array[a1] > array[a2]:
                array[a1].swap(array[a0])
                a1 -= 1
                a0 -= 1
                l1 -= 1
                if l1 == 0:
                    return 0
            else:
                array[a2].swap(array[a0])
                a2 -= 1
                a0 -= 1
                l2 -= 1
                if l2 == 0:
                    break
        res = l1
        while True:
            array[a1].swap(array[a0])
            a1 -= 1
            a0 -= 1
            l1 -= 1
            if not (l1 != 0):
                break
        return res

    def rmerge(this, array, a, l, r):
        i = 0
        while i < l:
            q = i
            j = i+r
            while j < l:
                if array[a+q] > array[a+j]:
                    q = j
                j += r
            if q != i:
                blockSwap(array, a+i, a+q, r)
            if i != 0:
                blockSwap(array, a+l, a+i, r)
                this.backmerge(array, a+(l+r-1), r, a+(i-1), r)
            i += r

    def rbnd(this, len):
        len //= 2
        k = 0
        i: int
        i = 1
        while i < len:
            i *= 2
            k += 1
        len //= k
        k = 1
        while k <= len:
            k *= 2
        return k

    def msort(this, array, a, len):
        if len < 12:
            this.sort(array, a, len)
            return
        r = this.rbnd(len)
        lr = (len//r-1)*r
        p = 2
        while p <= lr:
            compSwap(array, a+(p-2), a+(p-1))
            if (p & 2) != 0:
                p += 2
                continue
            blockSwap(array, a+(p-2), a+p, 2)
            m = len-p
            q = 2
            while True:
                q0 = 2*q
                if q0 > m or (p & q0) != 0:
                    break
                this.backmerge(array, a+(p-q-1), q, a+(p+q-1), q)
                q = q0
            this.backmerge(array, a+(p+q-1), q, a+(p-q-1), q)
            q1 = q
            q *= 2
            while (q & p) == 0:
                q *= 2
                this.rmerge(array, a+(p-q), q, q1)
            p += 2
        q1 = 0
        q = r
        while q < lr:
            if (lr & q) != 0:
                q1 += q
                if q1 != q:
                    this.rmerge(array, a+(lr-q1), q1, r)
            q *= 2
        s = len-lr
        this.msort(array, a+lr, s)
        blockSwap(array, a, a+lr, s)
        s += this.backmerge(array, a+(s-1), s, a+(lr-1), lr-s)
        this.msort(array, a, s)


@Sort("Merge Sorts", "Andrey Astrelin's In-Place Merge Sort", "Andrey's Merge")
def andreySortRun(array):
    AndreySort().msort(array, 0, len(array))
