import socket

HOST = '172.17.62.41'
PORT = 8080

print(f"Server in ascolto su porta {PORT}...")

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
    s.bind((HOST, PORT))
    s.listen()
    conn, addr = s.accept()
    print(f"Connesso da: {addr}")
    with conn:
        while True:
            data = conn.recv(1024)
            if not data: break
            # Stampa i dati grezzi appena arrivano
            print(data.decode('utf-8').strip())
