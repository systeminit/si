import { PubSubEngine } from 'graphql-subscriptions/dist/pubsub-engine';
import { connect, Client, ISubscriptionGrant, IClientPublishOptions, IClientSubscribeOptions } from 'mqtt';
import { PubSubAsyncIterator } from './pubsub-async-iterator';

export interface PubSubMQTTOptions {
  brokerUrl?: string;
  client?: Client;
  connectionListener?: (err: Error) => void;
  publishOptions?: PublishOptionsResolver;
  subscribeOptions?: SubscribeOptionsResolver;
  onMQTTSubscribe?: (id: number, granted: ISubscriptionGrant[]) => void;
  triggerTransform?: TriggerTransform;
  parseMessageWithEncoding?: string;
  rawData?: boolean;
}

export class MQTTPubSub implements PubSubEngine {

  private triggerTransform: TriggerTransform;
  private onMQTTSubscribe: SubscribeHandler;
  private subscribeOptionsResolver: SubscribeOptionsResolver;
  private publishOptionsResolver: PublishOptionsResolver;
  private mqttConnection: Client;
  private subscriptionMap: { [subId: number]: [string, Function] };
  private subsRefsMap: { [trigger: string]: Array<number> };
  private currentSubscriptionId: number;
  private parseMessageWithEncoding: string;
  private rawData: boolean;

  private static matches(pattern: string, topic: string) {
    const patternSegments = pattern.split('/');
    const topicSegments = topic.split('/');
    const patternLength = patternSegments.length;

    for (let i = 0; i < patternLength; i++) {
      const currentPattern = patternSegments[i];
      const currentTopic = topicSegments[i];
      if (!currentTopic && !currentPattern) {
        continue;
      }
      if (!currentTopic && currentPattern !== '#') {
        return false;
      }
      if (currentPattern[0] === '#') {
        return i === (patternLength - 1);
      }
      if (currentPattern[0] !== '+' && currentPattern !== currentTopic) {
        return false;
      }
    }
    return patternLength === (topicSegments.length);
  }

  constructor(options: PubSubMQTTOptions = {}) {
    this.triggerTransform = options.triggerTransform || (trigger => trigger as string);

    if (options.client) {
      this.mqttConnection = options.client;
    } else {
      const brokerUrl = options.brokerUrl || 'mqtt://localhost';
      this.mqttConnection = connect(brokerUrl);
    }

    this.mqttConnection.on('message', this.onMessage.bind(this));

    if (options.connectionListener) {
      this.mqttConnection.on('connect', options.connectionListener);
      this.mqttConnection.on('error', options.connectionListener);
    } else {
      this.mqttConnection.on('error', console.error);
    }

    this.subscriptionMap = {};
    this.subsRefsMap = {};
    this.currentSubscriptionId = 0;
    this.onMQTTSubscribe = options.onMQTTSubscribe || (() => null);
    this.publishOptionsResolver = options.publishOptions || (() => Promise.resolve({} as IClientPublishOptions));
    this.subscribeOptionsResolver = options.subscribeOptions || (() => Promise.resolve({} as IClientSubscribeOptions));
    this.parseMessageWithEncoding = options.parseMessageWithEncoding;
    this.rawData = options.rawData;
  }

  public publish(trigger: string, payload: any): boolean {
    this.publishOptionsResolver(trigger, payload).then(publishOptions => {
      let message;
      if (this.rawData) {
        message = Buffer.from(payload);
      } else {
        message = Buffer.from(JSON.stringify(payload), this.parseMessageWithEncoding);
      }

      this.mqttConnection.publish(trigger, message, publishOptions);
    });
    return true;
  }

  public subscribe(trigger: string, onMessage: Function, options?: Object): Promise<number> {
    const triggerName: string = this.triggerTransform(trigger, options);
    const id = this.currentSubscriptionId++;
    this.subscriptionMap[id] = [triggerName, onMessage];

    let refs = this.subsRefsMap[triggerName];
    if (refs && refs.length > 0) {
      const newRefs = [...refs, id];
      this.subsRefsMap[triggerName] = newRefs;
      return Promise.resolve(id);

    } else {
      return new Promise<number>((resolve, reject) => {
        // 1. Resolve options object
        this.subscribeOptionsResolver(trigger, options).then(subscriptionOptions => {

          // 2. Subscribing using MQTT
          this.mqttConnection.subscribe(triggerName, { qos: 0, ...subscriptionOptions }, (err, granted) => {
            if (err) {
              reject(err);
            } else {

              // 3. Saving the new sub id
              const subscriptionIds = this.subsRefsMap[triggerName] || [];
              this.subsRefsMap[triggerName] = [...subscriptionIds, id];

              // 4. Resolving the subscriptions id to the Subscription Manager
              resolve(id);

              // 5. Notify implementor on the subscriptions ack and QoS
              this.onMQTTSubscribe(id, granted);
            }
          });
        }).catch(err => reject(err));
      });
    }
  }

  public unsubscribe(subId: number) {
    const [triggerName = null] = this.subscriptionMap[subId] || [];
    const refs = this.subsRefsMap[triggerName];

    if (!refs) {
      throw new Error(`There is no subscription of id "${subId}"`);
    }

    let newRefs;
    if (refs.length === 1) {
      this.mqttConnection.unsubscribe(triggerName);
      newRefs = [];

    } else {
      const index = refs.indexOf(subId);
      if (index > -1) {
        newRefs = [...refs.slice(0, index), ...refs.slice(index + 1)];
      }
    }

    this.subsRefsMap[triggerName] = newRefs;
    delete this.subscriptionMap[subId];
  }

  public asyncIterator<T>(triggers: string | string[]): AsyncIterator<T> {
    return new PubSubAsyncIterator<T>(this, triggers);
  }

  private onMessage(topic: string, message: Buffer) {
    const subscribers = [].concat(
        ...Object.keys(this.subsRefsMap)
        .filter((key) => MQTTPubSub.matches(key, topic))
        .map((key) => this.subsRefsMap[key]),
    );

    // Don't work for nothing..
    if (!subscribers || !subscribers.length) {
      return;
    }

    let parsedMessage;
    if (this.rawData) {
      parsedMessage = {
        topic,
        message,
      };
    } else {
      const messageString = message.toString(this.parseMessageWithEncoding);
      try {
        parsedMessage = JSON.parse(messageString);
      } catch (e) {
        parsedMessage = messageString;
      }
    }

    for (const subId of subscribers) {
      const listener = this.subscriptionMap[subId][1];
      listener(parsedMessage);
    }
  }
}

export type Path = Array<string | number>;
export type Trigger = string | Path;
export type TriggerTransform = (trigger: Trigger, channelOptions?: Object) => string;
export type SubscribeOptionsResolver = (trigger: Trigger, channelOptions?: Object) => Promise<IClientSubscribeOptions>;
export type PublishOptionsResolver = (trigger: Trigger, payload: any) => Promise<IClientPublishOptions>;
export type SubscribeHandler = (id: number, granted: ISubscriptionGrant[]) => void;
