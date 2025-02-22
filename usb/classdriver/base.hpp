/**
 * @file usb/classdriver/base.hpp
 *
 * USB デバイスクラス用のドライバのベースクラス．
 */

#pragma once

#include "error.hpp"
#include "usb/endpoint.hpp"
#include "usb/setupdata.hpp"
#include <memory>

namespace usb
{
  class Device;

  class ClassDriver
  {
  public:
    ClassDriver(Device *dev);
    virtual ~ClassDriver();

    virtual std::unique_ptr<Error> Initialize() = 0;
    virtual std::unique_ptr<Error> SetEndpoint(const EndpointConfig &config) = 0;
    virtual std::unique_ptr<Error> OnEndpointsConfigured() = 0;
    virtual std::unique_ptr<Error> OnControlCompleted(EndpointID ep_id, SetupData setup_data,
                                                      const void *buf, int len) = 0;
    virtual std::unique_ptr<Error> OnInterruptCompleted(EndpointID ep_id, const void *buf, int len) = 0;

    /** このクラスドライバを保持する USB デバイスを返す． */
    Device *ParentDevice() const { return dev_; }

  private:
    Device *dev_;
  };
}
