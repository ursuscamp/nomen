def hash160(data):
    if type(data) == str:
        data = bytes.fromhex(data)
    import hashlib
    h = hashlib.new('sha256')
    h.update(data)
    d = h.digest()
    h = hashlib.new('ripemd160')
    h.update(d)
    return h.hexdigest()

