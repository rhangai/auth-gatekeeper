import { FastifyRequest, FastifyReply } from 'fastify';
import { ServerResponse } from 'http';

export type Request = FastifyRequest;
export type Reply = FastifyReply<ServerResponse>;
