def pairwiseSort(array, f, l, g):
    if f == l-g:
        return
    b = f+g
    while b < l:
        compSwap(array, b-g, b)
        b += 2*g
    if ((l-f)//g) % 2 == 0:
        pairwiseSort(array, f, l, g*2)
        pairwiseSort(array, f+g, l+g, g*2)
    else:
        pairwiseSort(array, f, l+g, g*2)
        pairwiseSort(array, f+g, l, g*2)
    a = 1
    while a < (l-f)//g:
        a = (a*2)+1
    b = f+g
    while b+g < l:
        c = a
        while c > 1:
            c //= 2
            if b+(c*g) < l:
                compSwap(array, b, b+(c*g))
        b += 2*g


@Sort("Concurrent Sorts", "Pairwise Sorting Network", "Pairwise")
def pairwiseSortRun(array):
    pairwiseSort(array, 0, len(array), 1)
