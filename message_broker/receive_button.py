import pika

credentials = pika.PlainCredentials('user', 'password')
parameters = pika.ConnectionParameters('localhost',
                                       5672,
                                       '/',
                                       credentials)
queue = 'DF:89:2B:DA:0B:CB/0000dad0-0000-0000-0000-000000000000/0000dad0-0001-0000-0000-000000000000'
def main():
    connection = pika.BlockingConnection(parameters)
    channel = connection.channel()

    def callback(ch, method, properties, body):
        print(f" [x] Received {body}")

    channel.basic_consume(queue=queue, on_message_callback=callback, auto_ack=True)

    print(' [*] Waiting for messages. To exit press CTRL+C')
    channel.start_consuming()

if __name__ == '__main__':
    main()
