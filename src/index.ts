import Fastify from 'fastify';

async function main() {
	const fastify = Fastify({ logger: true });

	fastify.get('/', async (request: any, reply: any) => {
		return { hello: 'world' };
	});

	try {
		await fastify.listen(3000);
		fastify.log.info(`server listening on ${fastify.server.address()}`);
	} catch (err) {
		fastify.log.error(err);
		process.exit(1);
	}
}

main();
