import socket
import threading
import time

# --- Part 1: 一个简易的模拟数据库服务器 ---
# 这个服务器会监听在本地的 9999 端口上。

def handle_client_connection(client_socket, address):
    """处理单个客户端连接的函数"""
    print(f"[服务器] 接受来自 {address} 的连接")
    try:
        while True:
            # 等待接收客户端发来的数据 (我们的 "SQL" 查询)
            request = client_socket.recv(1024).decode('utf-8')
            if not request:
                print(f"[服务器] 客户端 {address} 已断开")
                break
            
            print(f"[服务器] 收到查询: '{request}'")

            # 模拟数据库的逻辑：根据不同的查询返回不同的硬编码结果
            response = ""
            if request == "SELECT name FROM users WHERE id = 1;":
                response = "John Doe"
            elif request == "SELECT version();":
                response = "SimpleDB 1.0"
            else:
                response = "Error: Unknown SQL command"

            # 将响应数据发送回客户端
            client_socket.sendall(response.encode('utf-8'))
            print(f"[服务器] 已发送响应: '{response}'")

    except Exception as e:
        print(f"[服务器] 与 {address} 通信时发生错误: {e}")
    finally:
        client_socket.close()

def run_simple_db_server(host='127.0.0.1', port=9999):
    """启动服务器的主函数"""
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.bind((host, port))
    server.listen(5)
    print(f"[服务器] 正在监听端口 {port}...")

    # 持续接受客户端连接请求，并为每个连接创建一个新线程来处理
    # 这样服务器就不会被单个客户端阻塞
    while True:
        try:
            client_sock, address = server.accept()
            # 创建一个线程来处理客户端连接
            client_handler = threading.Thread(target=handle_client_connection, args=(client_sock, address))
            client_handler.start()
        except KeyboardInterrupt:
            print("[服务器] 服务器正在关闭...")
            break
        except Exception as e:
            print(f"[服务器] 发生错误: {e}")
            break
    server.close()


# --- Part 2: 我们的简易连接器 ---

class SimpleConnector:
    """
    一个模拟的数据库连接器类。
    """
    def __init__(self, host, port, user, password):
        # 实际的连接器会使用这些参数进行认证，这里我们仅作演示
        self._host = host
        self._port = port
        self._user = user
        self._password = password
        self._socket = None
        self._is_connected = False
        print(f"[连接器] 初始化完成，准备连接到 {host}:{port}")

    def connect(self):
        """连接到服务器"""
        try:
            # 创建一个 TCP/IP 套接字
            self._socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            # 连接到服务器
            self._socket.connect((self._host, self._port))
            self._is_connected = True
            print(f"[连接器] 成功连接到服务器！")
        except ConnectionRefusedError:
            print("[连接器] 连接失败：目标服务器拒绝连接。请确认服务器是否已启动。")
            self._is_connected = False
        except Exception as e:
            print(f"[连接器] 连接时发生未知错误: {e}")
            self._is_connected = False

    def execute(self, query):
        """执行一个 'SQL' 查询"""
        if not self._is_connected:
            print("[连接器] 错误：尚未建立连接，无法执行查询。")
            return None

        try:
            print(f"[连接器] -> 正在发送查询: '{query}'")
            # 1. 将查询字符串编码成字节流并发送
            self._socket.sendall(query.encode('utf-8'))
            
            # 2. 接收服务器返回的数据
            response = self._socket.recv(1024).decode('utf-8')
            print(f"[连接器] <- 收到响应: '{response}'")
            return response
        except Exception as e:
            print(f"[连接器] 执行查询时出错: {e}")
            self.close() # 如果出错，最好关闭连接
            return None

    def close(self):
        """关闭连接"""
        if self._socket:
            self._socket.close()
            self._is_connected = False
            print("[连接器] 连接已关闭。")
            
    # 实现 with 语句支持，可以自动关闭连接
    def __enter__(self):
        self.connect()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()


# --- Part 3: 主程序，演示如何使用连接器 ---

if __name__ == "__main__":
    # 1. 在一个后台线程中启动我们的模拟数据库服务器
    #    使用 daemon=True 意味着主程序退出时，该线程也会自动退出
    server_thread = threading.Thread(target=run_simple_db_server)
    server_thread.daemon = True
    server_thread.start()
    
    # 给服务器一点启动时间
    time.sleep(1) 
    print("\n--- 开始演示连接器 ---")

    # 2. 使用我们的连接器来连接服务器并执行操作
    # 使用 with 语句可以确保连接在使用后总是被关闭
    try:
        with SimpleConnector('127.0.0.1', 9999, user='admin', password='123') as conn:
            # 检查是否连接成功
            if conn._is_connected:
                # 执行一个服务器认识的查询
                user_name = conn.execute("SELECT name FROM users WHERE id = 1;")
                print(f"查询结果: User Name = {user_name}\n")
                
                # 执行另一个查询
                version = conn.execute("SELECT version();")
                print(f"查询结果: DB Version = {version}\n")

                # 执行一个服务器不认识的查询
                error_result = conn.execute("UPDATE users SET name = 'test';")
                print(f"查询结果: {error_result}\n")

    except Exception as e:
        print(f"客户端主程序出错: {e}")

    print("--- 演示结束 ---")